# Source Tree Analysis

```
project-root/
├── docs/                   # AI-generated documentation & knowledge base
├── scripts/                # Utility scripts (git hooks, setup scripts)
├── secrets/                # SOPS encrypted secret files
├── src/                    # Main Rust application source
│   ├── audit/              # Audit logging modules
│   ├── cli/                # Command-Line Interface handlers (clap)
│   ├── config/             # Application configuration (env, file parsing)
│   ├── engine/             # Core synchronization business logic
│   └── observability/      # Logging & tracing (tracing-subscriber)
└── tests/                  # Integration testing root
    ├── common/             # Shared test utilities
    ├── fixtures/           # Mock data and test responses
    ├── scripts/            # Shell scripts for e2e tests
    └── uat/                # User Acceptance Testing configurations
```

**Critical Folders Summary**
- Entry Point: `src/main.rs` (inferred from Rust conventions)
- Business Logic: `src/engine/` handles the JFrog API -> Nexus API sync.
- Operations: `src/cli/` drives the process; `tests/uat/` validates it against real instances.
