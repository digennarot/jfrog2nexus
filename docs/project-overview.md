# Project Overview

## Description
`jfrog2nexus` is a CLI-driven backend application written in Rust designed for migrating and synchronizing artifacts effectively from JFrog Artifactory to Sonatype Nexus.

## Executive Summary
This application provides robust, concurrent, and reliable synchronization capabilities by employing strong static typing, asynchronous I/O, and an embedded SQLite database for maintaining sync state and logging audit trails.

## Quick Reference
- **Architecture Type**: Monolith
- **Primary Language**: Rust
- **Key Frameworks**: Tokio, Axum, SQLx, Reqwest, Clap

## File Navigation
- [Architecture Overview](./architecture-root.md)
- [Source Tree Analysis](./source-tree-analysis.md)
- [Development Guide](./development-guide-root.md)
- [Deployment Guide](./deployment-guide.md)
- [Data Models](./data-models-root.md) (To be generated)
- [API Contracts](./api-contracts-root.md) (To be generated)
