---
number: 1055
title: "sdd: add QA section types — test-fixture, perf-test"
state: open
labels: [type:enhancement, priority:p3, crate:sdd]
group: "new-section-types"
---

# #1055 — sdd: add QA section types — test-fixture, perf-test

Parent: #1051

## Additional QA section types (lower priority)

### 1. `test-fixture`

| field | value |
|-------|-------|
| lang | `json` (JSON Schema) |
| skeleton output | mock data factory / seed files |
| note | `schema` partially covers this — evaluate overlap before implementing |

### 2. `perf-test`

| field | value |
|-------|-------|
| lang | `yaml` |
| skeleton output | k6 / Locust test file |

```yaml
perf_test:
  target: POST /orders
  profile:
    ramp_up: 30s
    steady: 5m
    ramp_down: 10s
  load:
    vus: 100
    rps: 500
  slo:
    p99_latency: 200ms
    error_rate: 0.1%
```
