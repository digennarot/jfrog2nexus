//! Test data factories and fixtures

pub struct ArtifactFactory {
    pub repo: String,
    pub path: String,
    pub name: String,
}

impl ArtifactFactory {
    pub fn new() -> Self {
        Self {
            repo: "test-local".to_string(),
            path: "org/example/pkg/".to_string(),
            name: "app-1.0.0.jar".to_string(),
        }
    }

    pub fn with_repo(mut self, repo: &str) -> Self {
        self.repo = repo.to_string();
        self
    }

    pub fn build(self) -> String {
        format!("{}/{}{}", self.repo, self.path, self.name)
    }
}

pub fn dummy_aql_response() -> serde_json::Value {
    serde_json::json!({
        "results": [
            {
                "repo": "test-local",
                "path": "org/example/pkg",
                "name": "app-1.0.0.jar",
                "size": 1024
            }
        ]
    })
}
