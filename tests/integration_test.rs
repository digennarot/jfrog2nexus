mod common;
mod fixtures;

use std::process::Command;
use wiremock::matchers::{method, path, query_param};
use wiremock::{Mock, ResponseTemplate};

#[tokio::test]
async fn test_migration_sync_flow() {
    if std::env::var("J2N_REAL_SERVICES").unwrap_or_default() == "true" {
        return;
    }

    // GIVEN: Mock servers are running and configured
    let ctx = common::TestContext::new().await;

    // 1. Mock JFrog Scanning
    let scan_response = serde_json::json!({
        "files": [
            {
                "uri": "/com/example/lib/1.0/lib-1.0.jar",
                "size": 13,
                "sha256": "f73674a39bca6f02d17026b49f30d42a1d0e45521de294f2a3053de470412f03",
                "folder": false
            }
        ]
    });

    Mock::given(method("GET"))
        .and(path("/api/storage/maven-local"))
        .and(query_param("list", ""))
        .and(query_param("deep", "1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(scan_response))
        .mount(ctx.jfrog_server.as_ref().unwrap())
        .await;

    // 2. Mock JFrog Download
    Mock::given(method("GET"))
        .and(path("/maven-local/com/example/lib/1.0/lib-1.0.jar"))
        .respond_with(ResponseTemplate::new(200).set_body_string("maven-content"))
        .mount(ctx.jfrog_server.as_ref().unwrap())
        .await;

    // 3. Mock Nexus Upload
    Mock::given(method("PUT"))
        .and(path(
            "/repository/maven-target/com/example/lib/1.0/lib-1.0.jar",
        ))
        .respond_with(ResponseTemplate::new(201))
        .mount(ctx.nexus_server.as_ref().unwrap())
        .await;

    // WHEN: We execute the jfrog2nexus CLI tool
    let mut cmd = Command::new("cargo");
    cmd.arg("run").arg("--");
    for arg in common::factories::mock_sync_args(&ctx.config_path)
        .into_iter()
        .skip(1)
    {
        cmd.arg(arg);
    }

    // Set necessary environment variables for tokens (since config doesn't include them for security)
    cmd.env("J2N_JFROG_TOKEN", "test-token");
    cmd.env("J2N_NEXUS_TOKEN", "test-token");
    cmd.env("J2N_ALLOW_HTTP", "true");
    cmd.env("RUST_LOG", "debug");

    let output = cmd.output().expect("Failed to execute jfrog2nexus sync");

    // THEN: The sync should execute successfully
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    if !output.status.success() {
        eprintln!("STDOUT:\n{}", stdout);
        eprintln!("STDERR:\n{}", stderr);
    }

    assert!(
        output.status.success(),
        "Command failed with status: {}",
        output.status
    );
    assert!(
        stderr.contains("Successfully transferred artifact")
            || stdout.contains("Successfully transferred artifact")
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// Multi-format migration flow: PyPI, npm, NuGet, Helm, Go, Raw
// Tests all 6 new package types end-to-end via wiremock + CLI subprocess.
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::test]
async fn test_migration_multi_format_flow() {
    if std::env::var("J2N_REAL_SERVICES").unwrap_or_default() == "true" {
        return;
    }

    use wiremock::MockServer;

    let jfrog_server = MockServer::start().await;
    let nexus_server = MockServer::start().await;

    // ── Structure of test data ──────────────────────────────────────────────
    // Each entry: (repo_type, jfrog_src, nexus_tgt, artifact_uri, body)
    let formats: &[(&str, &str, &str, &str, &str)] = &[
        (
            "pypi",
            "pypi-local",
            "pypi-target",
            "/packages/mylib/mylib-1.0-py3-none-any.whl",
            "wheel-content",
        ),
        (
            "npm",
            "npm-local",
            "npm-target",
            "/@myorg/mylib/-/mylib-1.0.0.tgz",
            "npm-tarball",
        ),
        (
            "nuget",
            "nuget-local",
            "nuget-target",
            "/mylib/1.0.0/mylib.1.0.0.nupkg",
            "nupkg-content",
        ),
        (
            "helm",
            "helm-local",
            "helm-target",
            "/myapp-1.0.0.tgz",
            "helm-chart",
        ),
        (
            "go",
            "go-local",
            "go-target",
            "/github.com/myorg/mylib/@v/v1.0.0.zip",
            "go-module",
        ),
        (
            "raw",
            "raw-local",
            "raw-target",
            "/binaries/myapp-linux-amd64",
            "binary-blob",
        ),
    ];

    // ── Register JFrog scan (file-list) mocks ───────────────────────────────
    for (_fmt, src, _tgt, uri, body) in formats {
        let scan_response = serde_json::json!({
            "files": [{
                "uri": uri,
                "size": body.len() as u64,
                "sha256": null,
                "folder": false
            }]
        });
        Mock::given(method("GET"))
            .and(path(format!("/api/storage/{}", src)))
            .and(query_param("list", ""))
            .and(query_param("deep", "1"))
            .respond_with(ResponseTemplate::new(200).set_body_json(scan_response))
            .mount(&jfrog_server)
            .await;

        // ── JFrog download ───────────────────────────────────────────────────
        let download_path = format!("/{}{}", src, uri);
        Mock::given(method("GET"))
            .and(path(download_path))
            .respond_with(ResponseTemplate::new(200).set_body_string(*body))
            .mount(&jfrog_server)
            .await;

        // ── Nexus upload ─────────────────────────────────────────────────────
        let upload_path = format!(
            "/repository/{}/{}",
            tgt_repo(_tgt, uri),
            uri.trim_start_matches('/')
        );
        Mock::given(method("PUT"))
            .and(path(upload_path))
            .respond_with(ResponseTemplate::new(201))
            .mount(&nexus_server)
            .await;
    }

    // ── Write multi-mapping config ───────────────────────────────────────────
    let config_path = format!("test_multi_{}.yaml", std::process::id());
    let mut mappings_yaml = String::new();
    for (fmt, src, tgt, _, _) in formats {
        mappings_yaml.push_str(&format!(
            "  - source: \"{}\"\n    target: \"{}\"\n    type: \"{}\"\n",
            src, tgt, fmt
        ));
    }
    let yaml = format!(
        "jfrog:\n  url: \"{}/\"\nnexus:\n  url: \"{}/\"\nmappings:\n{}",
        jfrog_server.uri(),
        nexus_server.uri(),
        mappings_yaml
    );
    std::fs::write(&config_path, yaml).expect("Failed to write multi-format test config");

    // ── Run the CLI ──────────────────────────────────────────────────────────
    let mut cmd = Command::new("cargo");
    cmd.arg("run")
        .arg("--")
        .arg("sync")
        .arg("--config")
        .arg(&config_path)
        .env("J2N_JFROG_TOKEN", "test-token")
        .env("J2N_NEXUS_TOKEN", "test-token")
        .env("J2N_ALLOW_HTTP", "true")
        .env("RUST_LOG", "debug");

    let output = cmd.output().expect("Failed to execute jfrog2nexus sync");
    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);

    let _ = std::fs::remove_file(&config_path);

    if !output.status.success() {
        eprintln!("STDOUT:\n{}", stdout);
        eprintln!("STDERR:\n{}", stderr);
    }

    assert!(
        output.status.success(),
        "Multi-format sync failed with status: {}",
        output.status
    );

    // Each artifact should have been successfully transferred (6 total)
    let transfer_count = stderr.matches("Successfully transferred artifact").count()
        + stdout.matches("Successfully transferred artifact").count();
    assert_eq!(
        transfer_count, 6,
        "Expected 6 successful transfers (one per format), got {}. STDERR:\n{}",
        transfer_count, stderr
    );
}

// Helper: return the target repo name for building Nexus PUT paths
fn tgt_repo<'a>(target_repo: &'a str, _uri: &str) -> &'a str {
    target_repo
}

#[tokio::test]
async fn test_real_services_flow() {
    if std::env::var("J2N_REAL_SERVICES").unwrap_or_default() != "true" {
        return;
    }

    // GIVEN: Real services are running and bootstrapped
    let ctx = common::TestContext::new().await;

    // Read tokens from files created during bootstrap
    let jfrog_token = "password"; // Default for admin
    let nexus_token = std::fs::read_to_string("tests/.nexus_password")
        .expect("Failed to read nexus password file")
        .trim()
        .to_string();

    // WHEN: We execute the sync for Maven and Docker
    let mut cmd = Command::new("cargo");
    cmd.arg("run")
        .arg("--")
        .arg("sync")
        .arg("--config")
        .arg(&ctx.config_path);

    cmd.env("J2N_JFROG_TOKEN", jfrog_token);
    cmd.env("J2N_NEXUS_TOKEN", nexus_token);
    cmd.env("J2N_ALLOW_HTTP", "true");
    cmd.env("RUST_LOG", "debug");

    let output = cmd.output().expect("Failed to execute jfrog2nexus sync");

    // THEN: Both Maven and Docker artifacts should be transferred
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    if !output.status.success() {
        eprintln!("STDOUT:\n{}", stdout);
        eprintln!("STDERR:\n{}", stderr);
    }

    assert!(output.status.success());
    // Maven check
    assert!(stderr.contains("Successfully transferred artifact") && stderr.contains("maven-local"));
    // Docker check
    assert!(
        stderr.contains("Successfully transferred artifact") && stderr.contains("docker-local")
    );
}
