# AGENTS.md

## What this file is for

This file describes the development workflow and conventions for mtop. It is read by agents to understand how to contribute effectively. It does not contain project-specific architecture or configuration; those live in `openspec/` and `ARCHITECTURE.md`.

> mtop is an agentic engineering project: developed by agents from idea to archive. Cleanroom-openspec is the methodology that keeps this workflow systematic, verifiable, and traceable.

## Methodology

**OpenSpec** ensures every feature has a formal ground truth before any code is written. Formal SHALL requirements define what the implementation must do, what tests must verify, and what auditors must confirm. Without this, implementation, testing, and verification are left to interpretation.

**Cleanroom** governs how that ground truth is produced. Before specifying anything, the team researches the problem domain and gathers input from all available sources. But all external information is treated as contaminated: it may carry implementation details, contradictions, or noise that would compromise the spec if carried over unchecked.

Two contamination boundaries are enforced:

- **Domain knowledge**: Only user-visible behaviors inform the spec. Implementation details, algorithms, and code patterns from any external source never appear in specs or source code.
- **Informal intent**: The maintainer's ideas and conversations are raw input, not ground truth. The Q&A phase clarifies and disambiguates intent. The Challenge phase tests for contradictions, feasibility gaps, and violations of prior decisions. Only after both phases does informal intent become a formal spec.

This separation (survey first, clean before you specify) is what makes the resulting spec reliable as ground truth for the full pipeline.

## Development workflow

mtop evolves through numbered **iterations**. Each iteration follows an 8-phase cycle:

1. **Idea**: Collect observations, feedback, or feature proposals; draft an idea file incrementally during Q&A
2. **Q&A**: Clarify scope and intent with the maintainer, one question at a time; update the idea file after each answer
3. **Challenge**: Adversarial review of the proposed changes (risk, feasibility, edge cases)
4. **Specify**: Write formal SHALL requirements with test scenarios
5. **Implement**: Agent pipeline: code from spec, write tests, review, verify; no ad-hoc changes outside the cycle
6. **Audit**: 3 gates: contamination check, spec compliance, final verification
7. **Review**: Project Review Board: 3 agents review code, conformance, and project completeness in parallel; each returns ACCEPT, REVISE, or REJECT
8. **Archive**: Append verified SHALLs to the living spec; commit retrospective

Maintainer feedback (from using the tool between iterations) enters the cycle as new idea files, not as a named phase.

## Project stages

Beyond individual iterations, the project moves through maturity stages, each representing a capability milestone (e.g., MVP, feature complete, open source release). Reaching a stage boundary triggers a cross-cutting review that spans the full codebase, not just the last iteration.

Known stages and their gates:

- **MVP**: Core functionality working end-to-end; foundational spec coverage in place
- **Open source**: Codebase reviewed for compliance with open-source standards before public release; scope is limited to compliance, no feature or content changes

## Living spec

`openspec/specs/` contains living specifications for each subsystem:

- `openspec/specs/tui-dashboard/spec.md`: TUI requirements (primary)
- `openspec/specs/metrics-collection/spec.md`
- `openspec/specs/cli-interface/spec.md`
- `openspec/specs/api-server/spec.md`

Each iteration appends its verified SHALL requirements to the relevant spec after the archive phase.

Planning artifacts supporting the agentic workflow (idea files, audit reports, retrospectives) are maintained separately.

## Roles

- **Maintainer**: Provides ideas, answers Q&A, gives final approval on Review Board verdicts
- **Architect**: Shapes ideas into formal specs, reviews implementation for correctness
- **Implementer**: Codes strictly from specs, no scope creep
- **Test engineer**: Writes and runs tests for every spec scenario
- **Auditor**: Verifies every SHALL against the implementation with evidence
- **Challenger**: Adversarial reviewer who identifies risks and gaps before implementation
- **Review board member**: One of three independent reviewers in Phase 7

## Agentic engineering roles

Each phase maps to a specific agent capability. Any agent with equivalent capabilities can participate:

| Phase | Agent type | What it does |
|-------|-----------|--------------|
| Idea / Q&A | Research and planning | Explores the codebase, surfaces relevant context, updates idea file incrementally after each answer |
| Challenge | Critic | Stress-tests assumptions, identifies edge cases, risks, and gaps before any spec is written |
| Specify | Architect | Translates intent into formal, testable SHALL requirements |
| Implement | Executor | Codes strictly from spec with no scope creep |
| Implement | Test engineer | Writes tests for every SHALL scenario; verifies all pass |
| Audit | Verifier (3 gates) | Contamination check, spec compliance, final build+test verification |
| Review | Code reviewer (x3, parallel) | Independent review of code, conformance to spec, and project completeness; returns ACCEPT/REVISE/REJECT |
| Archive | Writer | Appends verified SHALLs to living spec; commits retrospective |

## Conventions

- No code changes outside the iteration cycle
- Tests organized by feature/module, not by iteration number
- Panel and sub-panel titles use lowercase (even acronyms); body-content acronyms stay uppercase
- Maintainer feedback goes through the full idea-to-archive cycle; no ad-hoc changes
- Commits reference the iteration and SHALL requirement where applicable
- Before committing: `cargo fmt --check` must pass, `cargo clippy -- -D warnings` must be clean, and `cargo test` (lib and integration) must produce zero warnings in both build and test targets
