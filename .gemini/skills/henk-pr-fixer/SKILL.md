---
name: henk-pr-fixer
description: Specialized skill for analyzing Pull Request feedback and implementing best-practice solutions.
---

# PR Review Analyser & Fixer Workflow

This skill transforms Gemini CLI into an expert auditor that learns from the project's PR history and applies those learnings to the current codebase.

## Core Capabilities

1.  **Feedback Synthesis:** Aggregates comments from both open and closed Pull Requests to identify recurring issues or missed optimizations. **MANDATORY: Henk must always audit open PR comments and address them before any PR is considered ready for merge.**
2.  **Best-Practice Research:** Cross-references PR feedback with web research (e.g., Rust no-std standards, Intel HDA specs) to ensure solutions are industry-leading.
3.  **Autonomous Resolution:** Systematically implements fixes in the codebase following the project's strict Git strategy.

## Workflow

### 1. Extraction Phase
Execute the bundled script to retrieve the latest PR feedback in a token-efficient format:
```bash
bash scripts/fetch_pr_feedback.sh
```

### 2. Analysis Phase
- Identify specific code improvement requests (e.g., "missing bounds check", "inefficient heap usage").
- Categorize feedback by domain (Kernel, Audio, Networking, etc.).
- Search the codebase (`grep_search`) to locate the relevant files and lines.

### 3. Research Phase
For complex feedback, use the `google_web_search` tool to find the most current and robust implementation patterns. 
*Example: "Rust no-std chacha20poly1305 best practices"*

### 4. Implementation Phase
- Create a new granular branch: `git checkout -b fix/pr-<number>-<feedback-slug>`
- Apply surgical fixes using `replace` or `write_file`.
- Validate the changes locally using `./scripts/validate.sh`.
- Push and create a Pull Request.

### 5. Knowledge Persistence
Update `GEMINI.md` to reflect any new quality standards or safety rules derived from the PR feedback analysis.

## Mandatory Rules
- **Feedback First**: NEVER merge or finalize a task if there are unaddressed PR comments.
- **ZERO [allow]**: Never use `#[allow(...)]` to silence warnings. Fix the underlying issue.
- **Waker Safety**: Ensure no locks are used in Wakers or ISRs.
- **One Branch Per Feature**: Each fix must live in its own dedicated branch.
