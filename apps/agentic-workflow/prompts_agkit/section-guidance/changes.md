List files that will change. For MODIFY entries, include function/type-level targets:

```yaml
changes:
  - path: foo.rs
    action: CREATE
    description: new file
  - path: bar.rs
    action: MODIFY
    targets:
      - type: function
        name: handle_request
        change: add error handling
      - type: struct
        name: Config
        change: add timeout field
    do_not_touch: [validate_input, parse_args]
```

Target type values: function, struct, enum, trait, impl, method.
`targets` is required for MODIFY, optional for CREATE/DELETE.
`do_not_touch` lists functions/types the agent must NOT modify.

Begin with `<!-- type: changes lang: yaml -->`.
