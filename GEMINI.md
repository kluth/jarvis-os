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

### Workflow
1. Create a new branch from `main`: `git checkout -b feat/my-new-feature`
2. Implement and test your changes.
3. Commit using Conventional Commits.
4. Open a Pull Request to merge back into `main`.

## Voice-First Principles

JARVIS OS is a Voice-First Operating System. All new features should consider:
- Can this be controlled via voice?
- How does the system communicate state to the user without a screen?
- Is the audio pipeline preserved and prioritized?
