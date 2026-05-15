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

## Branching Strategy

To keep the `main` branch stable, all development must occur in dedicated branches.

- **Main Branch (`main`)**: Always stable and bootable.
- **Feature Branches (`feat/<feature-name>`)**: For new features.
- **Fix Branches (`fix/<bug-name>`)**: For bug fixes.
- **Refactor Branches (`refactor/<name>`)**: For code refactoring.

## Quality Standards & Pipeline

- **Zero Pipeline Failures**: A feature, fix, or task is **NEVER** considered finished if the CI pipeline fails. Stability is our highest priority.
- **Mandatory Local Validation**: Before pushing any changes, you MUST run the local validation script (`scripts/validate.sh`).
- **Resource Constraints**: Due to hardware limitations (Chromebook), `scripts/validate.sh` focuses on `cargo check` and `clippy`. Full `cargo build` is offloaded to the CI pipeline to preserve local resources.
- **Workflow Integrity**: ALWAYS test the run of the workflow and fix issues if they occur! ALWAYS! There are no exceptions.
- **Engineering Excellence**: NEVER use shortcuts. Every change must be super professional, perfectly clean, and well-structured.
- **Error Diagnostics**: When encountering compiler errors, always use `rustc --explain <error_code>` if suggested by the compiler. This ensures we follow Rust's best practices and deeply understand the root causes.

### Validation Framework
The project includes a multi-layer validation framework:
1. **`scripts/validate.sh`**: Central script for formatting, type-checking, and linting.
2. **Git Hook (`pre-push`)**: Automatically prevents pushing code that fails the validation script.
3. **CI Pipeline**: Conducts the final exhaustive build and test run in a specialized environment (QEMU).

### Workflow
1. Create a new branch from `main`: `git checkout -b feat/my-new-feature`
2. Implement and test your changes.
3. Run `scripts/validate.sh` locally (automatically triggered on `git push`).
4. Commit using Conventional Commits.
5. Open a Pull Request to merge back into `main`.

## Voice-First Principles

JARVIS OS is a Voice-First Operating System. All new features should consider:
- Can this be controlled via voice?
- How does the system communicate state to the user without a screen?
- Is the audio pipeline preserved and prioritized?
