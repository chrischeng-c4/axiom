---
id: mamba-stdlib-iteration
type: spec
title: "Stdlib: itertools and functools"
version: 1
spec_type: utility
created_at: 2026-02-22T11:20:41.468088+00:00
updated_at: 2026-02-22T11:20:41.468088+00:00
requirements:
  total: 10
  ids:
    - R1
    - R2
    - R3
    - R4
    - R5
    - R6
    - R7
    - R8
    - R9
    - R10
design_elements:
  has_mermaid: false
  has_json_schema: false
  has_pseudo_code: false
  has_api_spec: false
  has_semantic_diagrams: false
history:
  - timestamp: 2026-02-22T11:20:41.468088+00:00
    agent: "mcp"
    tool: "create_spec"
    action: "created"
---

<spec>

# Stdlib: itertools and functools

## Overview

Implement the itertools and functools stdlib modules for Mamba runtime. itertools provides combinatorial iterators (chain, islice, zip_longest, product, permutations, combinations, count, cycle, repeat, accumulate). functools provides higher-order function utilities (reduce, partial, lru_cache).

## Requirements

### R1 - itertools module registration

```yaml
id: R1
priority: high
status: draft
```

Create itertools_mod.rs with register(). Register attrs for chain, islice, zip_longest, product, permutations, combinations, count, cycle, repeat, accumulate, starmap.

### R2 - itertools.chain(a, b)

```yaml
id: R2
priority: high
status: draft
```

mb_itertools_chain: concatenate two iterables into one list.

### R3 - itertools.islice(iterable, stop) / islice(iterable, start, stop)

```yaml
id: R3
priority: high
status: draft
```

mb_itertools_islice: slice an iterable by index range, return list.

### R4 - itertools.product(a, b)

```yaml
id: R4
priority: medium
status: draft
```

mb_itertools_product: cartesian product of two iterables as list of tuples.

### R5 - itertools.permutations and combinations

```yaml
id: R5
priority: medium
status: draft
```

mb_itertools_permutations(iterable, r): all r-length permutations. mb_itertools_combinations(iterable, r): all r-length combinations.

### R6 - itertools.zip_longest(a, b)

```yaml
id: R6
priority: medium
status: draft
```

mb_itertools_zip_longest: zip iterables, filling shorter with None.

### R7 - itertools.accumulate(iterable)

```yaml
id: R7
priority: low
status: draft
```

mb_itertools_accumulate: running sum (or custom func) over iterable.

### R8 - functools module registration

```yaml
id: R8
priority: high
status: draft
```

Create functools_mod.rs with register(). Register attrs for reduce, partial, lru_cache.

### R9 - functools.reduce(func, iterable, initial)

```yaml
id: R9
priority: high
status: draft
```

mb_functools_reduce: left fold over iterable with binary function.

### R10 - functools.partial(func, *args)

```yaml
id: R10
priority: medium
status: draft
```

mb_functools_partial: return new callable with some arguments pre-filled. Store as closure with captured args.

## Acceptance Criteria

### Scenario: chain merges iterables

- **WHEN** itertools.chain([1,2], [3,4])
- **THEN** Returns [1, 2, 3, 4]

### Scenario: reduce sums list

- **WHEN** functools.reduce(add, [1,2,3,4], 0)
- **THEN** Returns 10

### Scenario: product gives cartesian pairs

- **WHEN** itertools.product([1,2], ['a','b'])
- **THEN** Returns [(1,'a'), (1,'b'), (2,'a'), (2,'b')]

</spec>
