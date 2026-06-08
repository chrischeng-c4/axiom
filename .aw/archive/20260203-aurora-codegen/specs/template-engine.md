---
id: template-engine
type: spec
title: "Tera Template Engine Integration"
version: 1
spec_type: algorithm
main_spec_ref: aurora-codegen-system
merge_strategy: replace
created_at: 2026-02-02T14:04:02.484488+00:00
updated_at: 2026-02-02T14:04:02.484488+00:00
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
    - type: class
      title: "TemplateEngine Class Diagram"
    - type: flowchart
      title: "Rendering Flow"
history:
  - timestamp: 2026-02-02T14:04:02.484488+00:00
    agent: "mcp"
    tool: "create_spec"
    action: "created"
  - timestamp: 2026-02-02T14:05:12.401037+00:00
    agent: "gemini:pro"
    tool: "revise_spec"
    action: "revised"
  - timestamp: 2026-02-02T14:05:29.091899+00:00
    agent: "codex:max"
    tool: "review_spec"
    action: "reviewed"---

<spec>

# Tera Template Engine Integration

## Overview

Provides the template engine functionality using Tera for the Aurora code generation system. It acts as the core rendering service, responsible for loading templates from the filesystem, registering custom string manipulation filters (e.g., PascalCase), and generating code output by combining templates with a data context.

## Requirements

### R1 - Tera Initialization

```yaml
id: R1
priority: high
status: draft
```

The TemplateEngine must initialize a Tera instance by loading all templates from a specified directory, supporting recursive discovery of .j2 or .tera files.

### R2 - Template Rendering

```yaml
id: R2
priority: high
status: draft
```

The system must provide a render method that accepts a template name and a serializable context object, returning the rendered string or an error.

### R3 - String Manipulation Filters

```yaml
id: R3
priority: medium
status: draft
```

The engine must register custom Tera filters for common case conversions: pascal_case, camel_case, snake_case, and kebab_case.

### R4 - Error Handling

```yaml
id: R4
priority: medium
status: draft
```

The system must return structured errors for: template not found, template syntax error, context type mismatch, and filter execution errors.

## Acceptance Criteria

### Scenario: Render Valid Template

- **GIVEN** A template 'test.rs.j2' with content '{{ name | pascal_case }}' and context '{ name: \"my_module\" }'
- **WHEN** render(\"test.rs.j2\", context) is called
- **THEN** Returns 'MyModule'

### Scenario: Render Missing Template

- **GIVEN** A non-existent template name 'ghost.j2'
- **WHEN** render(\"ghost.j2\", context) is called
- **THEN** Returns TemplateError::NotFound

### Scenario: Render Syntax Error

- **GIVEN** A template 'broken.j2' with invalid syntax '{{ unclosed tag'
- **WHEN** The engine initializes or attempts to render
- **THEN** Returns TemplateError::ParseError

### Scenario: Context Data Access

- **GIVEN** A template accessing '{{ config.version }}' and context '{ config: { version: \"1.0\" } }'
- **WHEN** render is called
- **THEN** Returns string containing '1.0'

## Diagrams

### TemplateEngine Class Diagram

```mermaid
classDiagram
    class TemplateEngine {
        <<service>>
        +new(PathBuf template_dir) TemplateEngine
        +render(String template_name, Context context) Result<String, Error>
        +register_filter(String name, Filter filter) void
    }
```

### Rendering Flow

```mermaid
flowchart LR
    Start((Start))
    LoadTemplates[Load Templates]
    RegisterFilters[Register Filters]
    RenderCall{Render(template, context)} 
    ProcessTemplate[Tera Processing]
    Output[Generated Code]
    Error[Return Error]
    Start -->|| LoadTemplates
    LoadTemplates -->|Success| RegisterFilters
    LoadTemplates -.->|Error| Error
    RegisterFilters -->|| RenderCall
    RenderCall -->|Valid Template| ProcessTemplate
    RenderCall -.->|Missing Template| Error
    ProcessTemplate -->|Success| Output
    ProcessTemplate -.->|Render Error| Error
```

</spec>
