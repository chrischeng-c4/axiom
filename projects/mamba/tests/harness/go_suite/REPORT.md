# go_suite baseline report

- captured: 2026-07-05 13:46:43 (host `MacBookPro`)
- platform: macOS-26.5.1-arm64-arm-64bit
- mamba: `target/release/mamba`
- python: `python3.12`
- go: NOT AVAILABLE on this host -- Go columns omitted below
- samples per shape/runtime: 5

## Epic #1071 bar

| Metric | Bar | Measured | Verdict |
|---|---|---|---|
| CPU geomean vs Go | <= 2.5x | n/a (no Go toolchain) | UNEVALUATED |
| Startup (time-to-first-output) | < 50.0ms | 33.324ms | PASS |
| Peak RSS geomean vs Go | <= 2.0x | n/a (no Go toolchain) | UNEVALUATED |

Fallback (informational only, NOT the epic bar): mamba/CPython CPU geomean = 5.28x.

## Per-shape table

| shape | mamba cpu (ms) | go cpu (ms) | py cpu (ms) | cpu vs go | mamba rss (MB) | go rss (MB) | rss vs go | checksums match |
|---|---|---|---|---|---|---|---|---|
| json_codec | 172.7 | n/a | 54.4 | n/a | 38.8 | n/a | n/a | yes |
| route_match | 196.4 | n/a | 22.6 | n/a | 98.8 | n/a | n/a | yes |
| data_transform | 280.0 | n/a | 32.9 | n/a | 78.1 | n/a | n/a | yes |
| template_render | 175.5 | n/a | 34.3 | n/a | 45.1 | n/a | n/a | yes |
| string_processing | 131.7 | n/a | 24.3 | n/a | 55.9 | n/a | n/a | yes |
| queue_pipeline | 102.1 | n/a | 30.6 | n/a | 51.5 | n/a | n/a | yes |

## Startup (hello shape, time-to-first-output)

| runtime | ttfb (ms) | cpu (ms) | rss (MB) |
|---|---|---|---|
| mamba | 33.324 | 30.9 | 30.4 |
| python | 16.32 | 14.6 | 12.2 |

