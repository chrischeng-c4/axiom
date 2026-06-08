---
id: mamba-stdlib-numeric
type: spec
title: "Stdlib: random, decimal, fractions"
version: 1
spec_type: utility
created_at: 2026-02-22T11:21:04.330999+00:00
updated_at: 2026-02-22T11:21:04.330999+00:00
requirements:
  total: 4
  ids:
    - R1
    - R2
    - R3
    - R4
design_elements:
  has_mermaid: false
  has_json_schema: false
  has_pseudo_code: false
  has_api_spec: false
  has_semantic_diagrams: false
history:
  - timestamp: 2026-02-22T11:21:04.330999+00:00
    agent: "mcp"
    tool: "create_spec"
    action: "created"
---

<spec>

# Stdlib: random, decimal, fractions

## Overview

Implement three numeric stdlib modules for Mamba: random (PRNG with randint, choice, shuffle, sample, uniform), decimal (arbitrary-precision decimal arithmetic wrapping Rust types), and fractions (rational number arithmetic with GCD-based simplification).

## Requirements

### R1 - random module registration

```yaml
id: R1
priority: high
status: draft
```

Create random_mod.rs with register(). Use xorshift64 PRNG in thread_local. Functions: mb_random_random (float 0..1), mb_random_randint(a,b), mb_random_choice(list), mb_random_shuffle(list), mb_random_sample(list,k), mb_random_uniform(a,b), mb_random_seed(n).

### R2 - random core functions

```yaml
id: R2
priority: high
status: draft
```

random() returns float [0,1). randint(a,b) returns int in [a,b]. choice(seq) picks random element. shuffle(list) permutes in-place. sample(list,k) returns k unique elements.

### R3 - decimal module

```yaml
id: R3
priority: medium
status: draft
```

Create decimal_mod.rs. mb_decimal_new(val) creates Decimal from string/int/float stored as string internally. Arithmetic: add, sub, mul, div via string-based decimal math. mb_decimal_quantize for rounding.

### R4 - fractions module

```yaml
id: R4
priority: medium
status: draft
```

Create fractions_mod.rs. mb_fraction_new(num, den) creates fraction as tuple(numerator, denominator) with GCD simplification. Arithmetic: add, sub, mul, div producing simplified fractions.

## Acceptance Criteria

### Scenario: randint returns in range

- **WHEN** random.randint(1, 6) called 100 times
- **THEN** All results in [1, 6]

### Scenario: fraction simplifies

- **WHEN** Fraction(4, 6)
- **THEN** Returns Fraction(2, 3)

</spec>
