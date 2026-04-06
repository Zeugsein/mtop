# OpenSpec Archive — Iteration 3

## Snapshot Metadata

- **Date**: 2026-04-06
- **Iteration**: 3
- **Spec count**: 4 (api-server, cli-interface, metrics-collection, tui-dashboard)
- **Total SHALL requirements**: 236
- **Test count**: 170 total (140 passing, 30 ignored)

## Spec Breakdown (SHALL counts)

| Spec               | SHALL count |
|--------------------|-------------|
| api-server         | 40          |
| cli-interface      | 32          |
| metrics-collection | 114         |
| tui-dashboard      | 50          |
| **Total**          | **236**     |

## Key Changes in Iteration 3 — 18 New Requirements

### metrics-collection (11 requirements)

| ID      | Title                                   |
|---------|-----------------------------------------|
| I3-C1   | Mach port cleanup in Sampler::drop      |
| I3-C2   | IOReport delta channel iteration        |
| I3-C3   | GPU channel group name                  |
| I3-C4   | Dynamic energy unit reading             |
| I3-C5   | Energy channel name matching            |
| I3-C6   | SMC endpoint targeting                  |
| I3-C7   | Apple Silicon flt temperature type      |
| I3-C8   | Dynamic SMC key enumeration             |
| I3-C9   | Debug sensor enumeration                |
| I3-C10  | Mach port count stability               |
| I3-C11  | IOReport subscription count stability   |

### api-server (3 requirements)

| ID      | Title                              |
|---------|------------------------------------|
| I3-S1   | Read timeout                       |
| I3-S2   | Per-IP connection limit            |
| I3-S3   | No sensitive information exposure  |

### tui-dashboard (3 requirements)

| ID      | Title                                      |
|---------|--------------------------------------------|
| I3-T1   | Sensor unavailable distinction             |
| I3-T2   | No sparkline growth for unavailable sensors|
| I3-T3   | Unavailable sensor display text            |

### cli-interface (1 requirement)

| ID      | Title                              |
|---------|------------------------------------|
| I3-L1   | Interval minimum clamping (100ms)  |

## Notes

Iteration 3 focused on correctness fixes for macOS hardware API integration: proper Mach port lifecycle management, accurate IOReport channel enumeration for GPU and power metrics, SMC sensor discovery, and defensive hardening of the HTTP server against slow-client attacks. The TUI gained explicit handling for sensors that return no data (N/A display, no phantom sparkline growth).
