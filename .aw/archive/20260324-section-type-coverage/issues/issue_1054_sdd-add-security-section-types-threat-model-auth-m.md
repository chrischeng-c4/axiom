---
number: 1054
title: "sdd: add security section types — threat-model, auth-matrix, security-test"
state: open
labels: [type:enhancement, priority:p2, crate:sdd]
group: "new-section-types"
---

# #1054 — sdd: add security section types — threat-model, auth-matrix, security-test

Parent: #1051

## Problem

Security role has no dedicated section types. Auth flows can use `state-machine`/`interaction` but lack security-specific semantics (trust boundaries, STRIDE, permission matrices).

## Proposal — 3 new section types

### 1. `threat-model`

| field | value |
|-------|-------|
| lang | `yaml` |
| skeleton output | security review checklist |

```yaml
threat_model:
  trust_boundaries:
    - name: public-internet
      components: [api-gateway]
    - name: internal-network
      components: [api-server, db]
  threats:
    - id: T1
      category: spoofing  # STRIDE
      target: api-gateway
      mitigation: JWT validation
```

### 2. `auth-matrix`

| field | value |
|-------|-------|
| lang | `yaml` |
| skeleton output | middleware / guard / policy code |

```yaml
auth_matrix:
  roles: [admin, editor, viewer]
  resources:
    orders:
      create: [admin, editor]
      read: [admin, editor, viewer]
      update: [admin, editor]
      delete: [admin]
```

### 3. `security-test`

| field | value |
|-------|-------|
| lang | `yaml` |
| skeleton output | security test suite |

```yaml
security_tests:
  - id: ST1
    category: injection  # OWASP
    target: POST /orders
    test: SQL injection in customer field
    payload: "'; DROP TABLE orders;--"
    expected: 400 Bad Request
```

### Section rules

```yaml
- match: "threat|attack surface|trust boundary|STRIDE"
  sections: [threat-model]
- match: "RBAC|ABAC|permission|role|access control|authorization matrix"
  sections: [auth-matrix]
- match: "OWASP|XSS|CSRF|injection|security test|pentest"
  sections: [security-test]
```
