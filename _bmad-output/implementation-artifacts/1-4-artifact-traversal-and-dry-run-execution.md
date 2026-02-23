# Story 1.4: Artifact Traversal and Dry-Run Execution

Status: done

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As an operations engineer,
I want to simulate a transfer to map all artifacts without moving data,
so that I can safely verify what will be migrated.

## Acceptance Criteria

1. **Given** a source repository with Docker and Maven artifacts
2. **When** I execute `jfrog2nexus sync --dry-run`
3. **Then** the tool queries the JFrog API using pagination tokens to recursively list all artifacts matching the mapping
4. **And** prints a simulated plan of what would be transferred, without downloading any bytes.

## Tasks / Subtasks

- [x] Task 1: Implement JFrog Artifact Scanner (AC: 1, 3)
  - [x] Implement recursive listing logic for Maven repositories via Artifactory File List API or AQL.
  - [x] Implement Docker manifest/tag listing logic for Docker repositories.
  - [x] Support pagination tokens for large repositories.
- [x] Task 2: Implement Dry-Run Logic (AC: 2, 4)
  - [x] Create a `Plan` struct to hold the list of artifacts to be transferred.
  - [x] Update `src/engine/mod.rs` to support a dry-run execution mode that only populates the `Plan`.
- [x] Task 3: Integrate into `sync` Command (AC: 2, 4)
  - [x] Update `src/cli/commands.rs` to handle the `sync` command with `--dry-run` flag.
  - [x] Print the plan in a clear, human-readable format.
- [x] Task 4: Unit/Integration Testing (AC: 3, 4)
  - [x] Mock the JFrog listing APIs to verify traversal and pagination.

## Dev Notes

### Technical Requirements
- **Recursion:** Maven repos can be deeply nested.
- **Docker Specifics:** Docker requires listing tags then manifests.
- **Memory Safety:** Don't hold all artifact metadata in memory if possible, use streams or iterators.

### Architecture Compliance
- **Engine Boundary:** Listing logic belongs in `src/engine/`.
- **Naming:** Follow `snake_case` for internal functions.

### Library / Framework Requirements
- `reqwest`
- `serde_json` (for parsing API responses)

### References
- PRD: FR7, FR9.
- Architecture: [Source: architecture.md#The Engine]
