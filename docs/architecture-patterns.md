# Architecture Patterns

## Root Application (CLI & Backend)
**Pattern:** Command-Line Interface (CLI) / Service-Oriented (Backend parts)

**Description:**
The application primarily functions as a CLI tool built with `clap`, orchestrating the synchronization process between JFrog and Nexus using `reqwest`. Concurrently, it embeds backend capabilities using `axum` and `sqlx` (SQLite), functioning as a local service or API provider. The architecture is a hybrid: a procedural CLI flow combined with a request-response API layer and a persistent data store.
