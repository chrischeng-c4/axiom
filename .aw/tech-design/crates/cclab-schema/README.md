# cclab-shield Specs

Pydantic-like validation with Rust performance.

## Overview

Shield provides data validation compatible with Pydantic's API but with Rust-level performance.

## Architecture

```mermaid
graph TB
    subgraph Shield["cclab-shield"]
        BaseModel[BaseModel]
        Field[Field Validators]
        Types[Type System]
        Serializer[Serializer]
        Settings[Settings Management]
    end

    BaseModel --> Field
    BaseModel --> Types
    BaseModel --> Serializer
    Settings --> BaseModel
```

## Validation Flow

```mermaid
flowchart TB
    Input[Input Data] --> Parse[Parse Input]
    Parse --> TypeCheck{Type Check}
    TypeCheck -->|Valid| FieldValidate[Field Validators]
    TypeCheck -->|Invalid| TypeError[Type Error]
    FieldValidate --> CustomValidate[Custom Validators]
    CustomValidate --> ModelValidate[Model Validators]
    ModelValidate -->|Valid| Output[Validated Model]
    ModelValidate -->|Invalid| ValidationError[Validation Error]
    FieldValidate -->|Invalid| ValidationError
```

## Type System (JSON Schema)

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "title": "Shield Type System",
  "definitions": {
    "FieldInfo": {
      "type": "object",
      "properties": {
        "default": { "description": "Default value" },
        "default_factory": { "type": "string", "description": "Factory function name" },
        "alias": { "type": "string" },
        "title": { "type": "string" },
        "description": { "type": "string" },
        "gt": { "type": "number" },
        "ge": { "type": "number" },
        "lt": { "type": "number" },
        "le": { "type": "number" },
        "min_length": { "type": "integer" },
        "max_length": { "type": "integer" },
        "pattern": { "type": "string" }
      }
    },
    "ValidatorInfo": {
      "type": "object",
      "properties": {
        "mode": { "enum": ["before", "after", "wrap", "plain"] },
        "check_fields": { "type": "boolean" }
      }
    }
  }
}
```

## Specs

| File | Type | Description |
|------|------|-------------|
| shield-basemodel-api-enhancement.md | algorithm | BaseModel API improvements |
| shield-ergonomic-validators.md | algorithm | Ergonomic validator syntax |
| shield-settings-management.md | data-model | Settings/config management |
