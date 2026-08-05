# Agent Instruction Sync Specification

## Purpose

Define a safe, reproducible source-of-truth and generated-target policy for repository instructions.

## Requirements

### Requirement: Canonical source and migration safety

The repository MUST track `.agents/AGENTS.md` as the sole canonical instruction source. Migration MUST preserve the reviewed content of the existing root `AGENTS.md`, MUST prevent source/target cycles, and MUST leave application behavior unchanged.

#### Scenario: Migrate existing instructions

- GIVEN a repository with an existing root `AGENTS.md`
- WHEN the change is applied
- THEN its reviewed content is merged into `.agents/AGENTS.md`
- AND each generated target resolves back to the canonical source without a cycle

#### Scenario: Detect unsafe migration

- GIVEN a target would become the canonical source or a symlink cycle is detected
- WHEN AgentSync is applied or checked
- THEN synchronization MUST fail without deleting the canonical source

### Requirement: Explicit target and MCP policy

AgentSync MUST declare explicit targets for the repository root (`AGENTS.md`), Claude (`CLAUDE.md`), and Copilot (`.github/copilot-instructions.md`). OpenCode MUST NOT receive a separate target because it consumes the root `AGENTS.md`; its only additional AgentSync surface is MCP, which is out of scope here. AgentSync MUST NOT generate an MCP target unless an MCP server is defined for this repository; adding a server later MUST require an explicit configuration change.

#### Scenario: Synchronize approved targets

- GIVEN AgentSync is installed and configuration is valid
- WHEN `agentsync apply` runs
- THEN only the approved root/Claude/Copilot destinations are reconciled

#### Scenario: No MCP server

- GIVEN no MCP server is defined
- WHEN synchronization runs
- THEN no MCP instruction destination is created

### Requirement: Managed generated destinations and ignore boundaries

Generated destinations MUST be ignored through AgentSync’s marker-managed block. Ordinary repository ignores such as `target/` MUST remain outside that block. AgentSync MUST NOT delete unrelated files or clean pre-existing untracked build output.

#### Scenario: Apply ignore policy

- GIVEN generated destinations and ordinary Rust artifacts are present
- WHEN AgentSync applies its ignore policy
- THEN generated destinations are covered by the managed block and `target/` remains independently ignored

#### Scenario: Roll back tooling

- GIVEN the tooling change is rolled back
- WHEN its files, generated destinations, and managed block are removed
- THEN the reviewed root `AGENTS.md` can be restored from version control without application or Cargo changes
