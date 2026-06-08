---
id: shield-settings-management
type: spec
title: "Shield Settings Management"
version: 1
spec_type: utility
created_at: 2026-01-28T07:41:49.369218+00:00
updated_at: 2026-01-28T07:41:49.369218+00:00
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
    - type: flowchart
      title: "Settings Loading Flow"
history:
  - timestamp: 2026-01-28T07:41:49.369218+00:00
    agent: "mcp"
    tool: "create_spec"
    action: "created"
---

<spec>

# Shield Settings Management

## Overview

This specification covers the implementation of Settings Management in cclab-shield, providing a BaseSettings class that can load configuration from environment variables and .env files.

## Requirements

### R1 - Load from Environment Variables

```yaml
id: R1
priority: medium
status: draft
```

BaseSettings should load values from environment variables by matching field names (case-insensitive by default).

### R2 - Support .env files

```yaml
id: R2
priority: medium
status: draft
```

Support loading from .env files using the dotenvy crate in Rust.

### R3 - Environment Prefix Support

```yaml
id: R3
priority: medium
status: draft
```

Allow defining an environment variable prefix (e.g., APP_) to avoid collisions.

### R4 - Validation Integration

```yaml
id: R4
priority: medium
status: draft
```

Settings should be validated using the same engine as BaseModel.

## Acceptance Criteria

### Scenario: Load from ENV with prefix

- **GIVEN** An environment variable APP_DATABASE_URL=postgres://localhost:5432 is set.
- **WHEN** A Settings class with env_prefix='APP_' is instantiated.
- **THEN** The settings instance should have database_url attribute set correctly.

### Scenario: Load from .env file

- **GIVEN** A .env file with DEBUG=true is present.
- **WHEN** The Settings class is instantiated.
- **THEN** The settings instance should have debug=True.

### Scenario: Ignore extra env vars

- **GIVEN** An env var MY_SETTING=value is set but not defined in the class.
- **WHEN** The Settings class is instantiated.
- **THEN** The extra variable should be ignored without error.

### Scenario: Validation failure in settings

- **GIVEN** An env var PORT=abc is set for an int field.
- **WHEN** The Settings class is instantiated.
- **THEN** A ValidationError should be raised during instantiation.

## Diagrams

### Settings Loading Flow

```mermaid
flowchart LR
    init[Python User Code]
    python_init[BaseSettings.__init__]
    rust_loader[Rust SettingsLoader]
    dotenvy_call[dotenvy::from_path]
    env_vars_call[std::env::vars]
    return_values[Return Value::Object]
    init -->|Instantiate BaseSettings subclass| python_init
    python_init -->|Call Rust load_settings| rust_loader
    rust_loader -->|Read .env file| dotenvy_call
    rust_loader -->|Read environment variables| env_vars_call
    rust_loader,dotenvy_call,env_vars_call -->|Merge and Validate| return_values
```

</spec>
