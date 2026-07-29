# Mamba ownership-site audit

`audit.py` inventories every list, set, and tuple constructor that owns or
retains `MbValue` elements under `projects/mamba/src/runtime/`.

```bash
python3 projects/mamba/tools/ownership/audit.py
python3 projects/mamba/tools/ownership/audit.py --check
python3 projects/mamba/tools/ownership/audit.py --fixture /tmp/case.rs
```

The default command emits one stable JSON document. Rows are ordered by
`site_id`; the inventory digest excludes presentation-only line numbers.
`site_id` is derived from repository-relative path, enclosing function,
constructor, and the resolved local-origin fingerprint. Local variable names
are not part of that fingerprint.

The provenance classes are:

- `OWNED`: every element is proven fresh, retained, or immediate.
- `BORROWED`: elements come from a parameter, container copy, or a
  conservatively classified complete expression.
- `MIXED`: both proven-owned and borrowed origins reach the constructor.
- `UNCLASSIFIED`: parsing was incomplete or a call target could not be
  established. This is never converted to `OWNED`.

Each row separately records the constructor contract:
`CONSUMES_OWNED` for ownership-taking constructors and `RETAINS_BORROWED` for
their retaining twins. The provenance class is derived from the argument; it
is not inferred from the constructor name.

`--check` fails on an empty inventory, diagnostics, truncated syntax, or any
current-tree `UNCLASSIFIED` row. It also proves byte-identical unchanged-tree
output, row/count reconciliation, exact constructor matching, comment/string
masking, nested delimiter handling, rename-stable origin identity, and dynamic
recomputation of an unseen temporary fixture.
