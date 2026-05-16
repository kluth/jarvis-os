# JARVIS OS - Development Guidelines

This document outlines the engineering standards and workflows for JARVIS OS.

## Versioning & Commits

We follow [Conventional Commits](https://www.conventionalcommits.org/). This enables automated changelog generation and clear project history.

### Commit Format
`<type>[optional scope]: <description>`

**Types:**
- `feat`: A new feature (corresponds to MINOR in Semantic Versioning).
- `fix`: A bug fix (corresponds to PATCH in Semantic Versioning).
- `docs`: Documentation only changes.
- `style`: Changes that do not affect the meaning of the code (white-space, formatting, etc).
- `refactor`: A code change that neither fixes a bug nor adds a feature.
- `perf`: A code change that improves performance.
- `test`: Adding missing tests or correcting existing tests.
- `chore`: Changes to the build process or auxiliary tools and libraries.

**Example:**
`feat(audio): implement HDA controller initialization`

## Strict Git & Branching Strategy

To maintain a pristine, easily reviewable, and perfectly linear history, we enforce the following strict Git workflow. **"One-for-all" branches are strictly prohibited.**

- **Main Branch (`main`)**: The single source of truth. Always stable, always perfectly clean, and heavily protected.
- **Granular Feature Branches (`feat/<specific-scope>`)**: Branches MUST be hyper-focused (e.g., `feat/audio-hda-driver`, not `feat/jarvis-capabilities`).
- **Atomic Commits**: Every commit MUST represent a single, isolated, logical change. Do not mix refactoring with new features or CI fixes in the same commit.
- **Mandatory Pull Requests**: All changes into `main` must be merged via a Pull Request. Direct pushes to `main` are strictly forbidden.
- **Linear History & Cleanup**: Merge commits are forbidden. Branches must be rebased against `main` and merged using "Squash and Merge". Branches must be deleted immediately after merging.

### Workflow
1. Create a granular branch: `git checkout -b feat/my-specific-feature`
2. Implement and test your changes.
3. Run `scripts/validate.sh` locally.
4. Commit using Conventional Commits.
5. Open a Pull Request for review and merge.

## Quality Standards & Pipeline

- **Zero Pipeline Failures**: A feature, fix, or task is **NEVER** considered finished if the CI pipeline fails. Stability is our highest priority.
- **Mandatory Local Validation**: Before pushing any changes, you MUST run the local validation script (`scripts/validate.sh`).
- **Resource Constraints**: Due to hardware limitations (Chromebook), `scripts/validate.sh` focuses on `cargo check` and `clippy`. Full `cargo build` is offloaded to the CI pipeline to preserve local resources.
- **Workflow Integrity**: ALWAYS test the run of the workflow and fix issues if they occur! ALWAYS! There are no exceptions.
- **Engineering Excellence**: NEVER use shortcuts. Every change must be super professional, perfectly clean, and well-structured.
- **Issue-First Policy**: EVERY new idea, feature, or architectural change MUST be documented as a fully fleshed-out GitHub Issue BEFORE any implementation begins. IMMER!
- **Issue Management**: GitHub Issues MUST be kept up-to-date at all times. Stale issues ("Karteileichen") are strictly prohibited. Every open issue must reflect an active, prioritized task or a currently reproducible bug.
- **Waker & ISR Safety**: NEVER use code that requires locks (e.g., `println!`, `WRITER.lock()`) inside Wakers or Interrupt Service Routines. This leads to immediate deadlocks. Use lock-free diagnostics (atomics) instead.
- **No Magic Numbers**: Avoid hardcoded array bounds or numeric constants without a named constant (`const`). Tie array sizes to enums or common constants to ensure type safety and scalability.
- **Error Diagnostics**: When encountering compiler errors, always use `rustc --explain <error_code>` if suggested by the compiler. This ensures we follow Rust's best practices and deeply understand the root causes.

## Agent Operational Mandates

To ensure system integrity during autonomous development, all agents (e.g., Henk) must follow these strict operational rules:

- **Real-Time CI Verification**: No task is considered "done" until the agent has verified the successful completion of the GitHub Actions workflow for the relevant PR. ALWAYS check the run status.
- **Autonomous Loop**: Agents must operate in an iterative loop: `Scan/Research -> Plan -> Implement -> Local Validate -> PR -> Verify CI -> Merge -> Repeat`.
- **Zero Regression Policy**: If a PR causes a CI failure, the agent must prioritize the fix immediately before starting any other task.

### Validation Framework
The project includes a multi-layer validation framework:
1. **`scripts/validate.sh`**: Central script for formatting, type-checking, and linting.
2. **Git Hook (`pre-push`)**: Automatically prevents pushing code that fails the validation script.
3. **CI Pipeline**: Conducts the final exhaustive build and test run in a specialized environment (QEMU).

## Voice-First Principles

JARVIS OS is a Voice-First Operating System. All new features should consider:
- Can this be controlled via voice?
- How does the system communicate state to the user without a screen?
- Is the audio pipeline preserved and prioritized?
