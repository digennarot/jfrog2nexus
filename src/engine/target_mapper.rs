use crate::config::RepoType;
use crate::engine::scanner::Artifact;

pub enum TargetApiAction {
    Put { url: String },
    DockerBlob { name: String, digest: String },
    DockerManifest { name: String, reference: String },
}

pub struct TargetMapper;

impl TargetMapper {
    /// Build a simple `PUT repository/{target_repo}/{path}` action — used by all
    /// non-Docker format types (Maven, PyPI, npm, NuGet, Helm, Go, Raw).
    fn simple_put_action(artifact: &Artifact) -> TargetApiAction {
        let path = artifact.path.trim_start_matches('/');
        TargetApiAction::Put {
            url: format!("repository/{}/{}", artifact.target_repo, path),
        }
    }

    pub fn map_artifact(artifact: &Artifact) -> TargetApiAction {
        match artifact.repo_type {
            // ── Non-Docker formats all use the simple PUT path ──────────────
            RepoType::Maven
            | RepoType::Pypi
            | RepoType::Npm
            | RepoType::Nuget
            | RepoType::Helm
            | RepoType::Go
            | RepoType::Raw => Self::simple_put_action(artifact),

            // ── Docker: content-addressed blobs and signed manifests ─────────
            RepoType::Docker => {
                // Heuristic for Docker V2 mapping based on Artifactory storage layout:
                //   <image>/<tag>/manifest.json  →  DockerManifest
                //   <image>/_/sha256__<hex>       →  DockerBlob
                let parts: Vec<&str> = artifact.path.trim_start_matches('/').split('/').collect();

                if parts.len() >= 3 {
                    let image_name = parts[..parts.len() - 2].join("/");
                    let second_to_last = parts[parts.len() - 2];
                    let file_name = parts[parts.len() - 1];

                    if file_name == "manifest.json" {
                        return TargetApiAction::DockerManifest {
                            name: image_name,
                            reference: second_to_last.to_string(), // tag
                        };
                    } else if second_to_last == "_" {
                        // Artifactory stores blobs as sha256__<hex>; registry uses sha256:<hex>
                        let digest = file_name.replace("__", ":");
                        return TargetApiAction::DockerBlob {
                            name: image_name,
                            digest,
                        };
                    }
                }

                // Fallback: avoid panicking on unexpected layout
                Self::simple_put_action(artifact)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::RepoType;
    use crate::engine::scanner::Artifact;

    fn make_artifact(repo_type: RepoType, path: &str) -> Artifact {
        Artifact {
            source_repo: "src-repo".to_string(),
            target_repo: "tgt-repo".to_string(),
            path: path.to_string(),
            size: 1,
            sha256: None,
            repo_type,
        }
    }

    fn put_url(action: TargetApiAction) -> String {
        match action {
            TargetApiAction::Put { url } => url,
            _ => panic!("Expected Put action"),
        }
    }

    // ── Maven ────────────────────────────────────────────────────────────────

    #[test]
    fn test_map_maven() {
        let a = make_artifact(RepoType::Maven, "/com/example/lib/1.0/lib-1.0.jar");
        let url = put_url(TargetMapper::map_artifact(&a));
        assert_eq!(url, "repository/tgt-repo/com/example/lib/1.0/lib-1.0.jar");
    }

    // ── PyPI ─────────────────────────────────────────────────────────────────

    #[test]
    fn test_map_pypi_wheel() {
        let a = make_artifact(RepoType::Pypi, "/packages/mylib/mylib-1.0-py3-none-any.whl");
        let url = put_url(TargetMapper::map_artifact(&a));
        assert_eq!(
            url,
            "repository/tgt-repo/packages/mylib/mylib-1.0-py3-none-any.whl"
        );
    }

    #[test]
    fn test_map_pypi_sdist() {
        let a = make_artifact(RepoType::Pypi, "/packages/mylib/mylib-1.0.tar.gz");
        let url = put_url(TargetMapper::map_artifact(&a));
        assert_eq!(url, "repository/tgt-repo/packages/mylib/mylib-1.0.tar.gz");
    }

    // ── npm ──────────────────────────────────────────────────────────────────

    #[test]
    fn test_map_npm_scoped() {
        let a = make_artifact(RepoType::Npm, "/@myorg/mylib/-/mylib-1.0.0.tgz");
        let url = put_url(TargetMapper::map_artifact(&a));
        assert_eq!(url, "repository/tgt-repo/@myorg/mylib/-/mylib-1.0.0.tgz");
    }

    #[test]
    fn test_map_npm_unscoped() {
        let a = make_artifact(RepoType::Npm, "/mylib/-/mylib-2.3.1.tgz");
        let url = put_url(TargetMapper::map_artifact(&a));
        assert_eq!(url, "repository/tgt-repo/mylib/-/mylib-2.3.1.tgz");
    }

    // ── NuGet ────────────────────────────────────────────────────────────────

    #[test]
    fn test_map_nuget() {
        let a = make_artifact(RepoType::Nuget, "/mylib/1.0.0/mylib.1.0.0.nupkg");
        let url = put_url(TargetMapper::map_artifact(&a));
        assert_eq!(url, "repository/tgt-repo/mylib/1.0.0/mylib.1.0.0.nupkg");
    }

    // ── Helm ─────────────────────────────────────────────────────────────────

    #[test]
    fn test_map_helm() {
        let a = make_artifact(RepoType::Helm, "/myapp-1.0.0.tgz");
        let url = put_url(TargetMapper::map_artifact(&a));
        assert_eq!(url, "repository/tgt-repo/myapp-1.0.0.tgz");
    }

    // ── Go ───────────────────────────────────────────────────────────────────

    #[test]
    fn test_map_go_module_zip() {
        let a = make_artifact(RepoType::Go, "/github.com/myorg/mylib/@v/v1.0.0.zip");
        let url = put_url(TargetMapper::map_artifact(&a));
        assert_eq!(
            url,
            "repository/tgt-repo/github.com/myorg/mylib/@v/v1.0.0.zip"
        );
    }

    #[test]
    fn test_map_go_module_info() {
        let a = make_artifact(RepoType::Go, "/github.com/myorg/mylib/@v/v1.0.0.info");
        let url = put_url(TargetMapper::map_artifact(&a));
        assert_eq!(
            url,
            "repository/tgt-repo/github.com/myorg/mylib/@v/v1.0.0.info"
        );
    }

    // ── Raw / Generic ────────────────────────────────────────────────────────

    #[test]
    fn test_map_raw_binary() {
        let a = make_artifact(RepoType::Raw, "/binaries/myapp-linux-amd64");
        let url = put_url(TargetMapper::map_artifact(&a));
        assert_eq!(url, "repository/tgt-repo/binaries/myapp-linux-amd64");
    }

    // ── Docker ───────────────────────────────────────────────────────────────

    #[test]
    fn test_map_docker_manifest() {
        let a = make_artifact(
            RepoType::Docker,
            "/library/hello-world/latest/manifest.json",
        );
        match TargetMapper::map_artifact(&a) {
            TargetApiAction::DockerManifest { name, reference } => {
                assert_eq!(name, "library/hello-world");
                assert_eq!(reference, "latest");
            }
            _ => panic!("Expected DockerManifest"),
        }
    }

    #[test]
    fn test_map_docker_blob() {
        let a = make_artifact(RepoType::Docker, "/library/hello-world/_/sha256__abcdef");
        match TargetMapper::map_artifact(&a) {
            TargetApiAction::DockerBlob { name, digest } => {
                assert_eq!(name, "library/hello-world");
                assert_eq!(digest, "sha256:abcdef");
            }
            _ => panic!("Expected DockerBlob"),
        }
    }

    #[test]
    fn test_map_docker_fallback_short_path() {
        // Only 2 path segments → fallback to simple PUT
        let a = make_artifact(RepoType::Docker, "/myimage/layer");
        let url = put_url(TargetMapper::map_artifact(&a));
        assert!(url.starts_with("repository/tgt-repo/"));
    }
}
