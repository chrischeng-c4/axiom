---
id: cli-architecture
type: spec
title: "Taipan CLI Integration"
version: 1
spec_type: integration
main_spec_ref: cclab-cli/architecture
merge_strategy: extend
created_at: 2026-02-12T07:42:59.426432+00:00
updated_at: 2026-02-12T07:42:59.426432+00:00
requirements:
  total: 4
  ids:
    - R1
    - R2
    - R3
    - R4
design_elements:
  has_mermaid: true
  has_json_schema: false
  has_pseudo_code: false
  has_api_spec: false
  has_semantic_diagrams: false
  diagrams:
    - type: sequence
      title: "Taipan Run Sequence"
history:
  - timestamp: 2026-02-12T07:42:59.426432+00:00
    agent: "mcp"
    tool: "create_spec"
    action: "created"
---

<spec>

# Taipan CLI Integration

## Overview

This specification covers the integration of the Taipan compiler into the unified Cclab CLI. It defines the 'taipan' subcommand, its registration via the modular CLI architecture, and the command flow for compilation and execution.

## Requirements

### R1 - CliModule Registration

```yaml
id: R1
priority: high
status: draft
```

Register the TaipanCli module with the unified CLI using the linkme-based distributed slice pattern.

### R2 - Taipan Compile Command

```yaml
id: R2
priority: high
status: draft
```

Provide a 'compile' subcommand to transform Taipan source files into native executables.

### R3 - Pluggable Backend Support

```yaml
id: R3
priority: medium
status: draft
```

Support a '--backend' argument to select the codegen backend, defaulting to 'cranelift'.

### R4 - Taipan Run Command

```yaml
id: R4
priority: high
status: draft
```

Provide a 'run' subcommand that compiles and executes Taipan source code in a single step.

## Acceptance Criteria

### Scenario: Successful CLI Registration

- **WHEN** The CLI is built and 'cclab' is executed.
- **THEN** The 'taipan' subcommand should be listed in 'cclab --help' and correctly dispatched.

### Scenario: Compile Source File with Default Backend

- **WHEN** 'cclab taipan compile hello.tp -o hello' is executed.
- **THEN** A native binary should be produced using the Cranelift backend.

### Scenario: Compile with Specific Backend

- **WHEN** 'cclab taipan compile hello.tp --backend cranelift' is executed.
- **THEN** The compiler should use the specified backend (e.g., cranelift) for code generation.

### Scenario: Compile and Run

- **WHEN** 'cclab taipan run hello.tp' is executed.
- **THEN** The program should be compiled, executed, and its output displayed to the user.

## Diagrams

### Taipan Run Sequence

```mermaid
sequenceDiagram
    actor User as User
    participant CclabCli as Cclab CLI
    participant TaipanCli as Taipan CLI Module
    participant CompilerCore as Compiler Core
    participant BackendCranelift as Cranelift Backend
    participant Executable as Target Executable
    User->CclabCli: cclab taipan run main.tp
    CclabCli->TaipanCli: execute(matches)
    TaipanCli->CompilerCore: compile(file)
    CompilerCore->BackendCranelift: generate_code(ir)
    BackendCranelift-->CompilerCore: return binary
    CompilerCore-->TaipanCli: return executable_path
    TaipanCli->Executable: spawn()
    Executable-->User: output
```

</spec>
