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
---
id: jet-install-sh-marker-syntax
entry: issue
nodes:
  issue: { kind: start, label: "Jet install.sh syntax failure" }
  marker: { kind: process, label: "HANDWRITE wrapper uses //" }
  parser: { kind: process, label: "POSIX sh parses line 2 before installer logic" }
  fail: { kind: terminal, label: "Installer exits with syntax error" }
  fix: { kind: process, label: "Use shell comment marker #" }
  valid: { kind: terminal, label: "sh -n projects/jet/install.sh passes" }
edges:
  - { from: issue, to: marker }
  - { from: marker, to: parser }
  - { from: parser, to: fail }
  - { from: marker, to: fix }
  - { from: fix, to: valid }
---
flowchart TD
    issue([Jet install.sh syntax failure]) --> marker[HANDWRITE wrapper uses //]
    marker --> parser[POSIX sh parses line 2 before installer logic]
    parser --> fail([Installer exits with syntax error])
    marker --> fix[Use shell comment marker #]
    fix --> valid([sh -n projects/jet/install.sh passes])
```

## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: jet-install-sh-marker-syntax-verification
requirements:
  repo_install_scripts:
    id: R2
    text: "Every install.sh in the repository remains parseable by POSIX sh after the Jet marker repair."
    kind: regression
    risk: medium
    verify: find . -name install.sh -exec sh -n {} ; -print
  shell_syntax:
    id: R1
    text: "projects/jet/install.sh uses shell-comment HANDWRITE markers so POSIX sh can parse the installer before executing any bootstrap logic."
    kind: regression
    risk: high
    verify: sh -n projects/jet/install.sh
---
flowchart TD
    r1[R1 shell syntax] --> sh_n_projects_jet_install_sh[sh -n projects/jet/install.sh]
    r2[R2 repo install scripts] --> find_name_install_sh_exec_sh_n_print[find . -name install.sh -exec sh -n {} ; -print]
```
