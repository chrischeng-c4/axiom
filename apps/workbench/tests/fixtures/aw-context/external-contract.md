<!-- HANDWRITE-BEGIN gap="missing-generator:unit-test:74db439f" tracker="pending-tracker" reason="Minimal typed EC fixture with assertions and a verifier command." -->
---
id: fixture-aw-context-ec
kind: external-contract
---

# Fixture AW Context External Contract

## Assertions

- id: response_ok
  given: a configured read-only context request
  then: the typed artifact is rendered without source mutation

## Verifier

```bash
aw ec verify --project workbench
```

## Relationships

Design source: `tech-design.md`.
<!-- HANDWRITE-END -->
