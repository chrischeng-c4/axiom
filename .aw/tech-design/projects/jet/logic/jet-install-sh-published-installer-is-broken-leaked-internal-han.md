---
id: projects-jet-logic-jet-install-sh-published-installer-is-broken-leaked-internal-han-md
fill_sections: [logic, unit-test, changes]
capability_refs:
  - id: rust-native-frontend-toolchain
    role: primary
    claim: production-replacement-readiness
    coverage: partial
    rationale: "The published Jet installer must be valid POSIX sh before users can bootstrap the Jet frontend toolchain."
---

# jet install.sh: published installer marker syntax

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
flowchart TD
    issue[Jet install.sh syntax failure] --> marker[HANDWRITE wrapper uses //]
    marker --> parser[POSIX sh parses line 2 before any installer logic]
    parser --> fail[Installer exits with syntax error]
    marker --> fix[Use shell comment marker #]
    fix --> valid[sh -n projects/jet/install.sh passes]
```
