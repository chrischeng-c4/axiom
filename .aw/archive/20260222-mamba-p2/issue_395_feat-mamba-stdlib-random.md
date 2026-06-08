---
number: 395
title: "feat(mamba): stdlib random"
state: open
labels: [enhancement, P2, crate:mamba]
---

# #395 — feat(mamba): stdlib random

## Summary
Implement `random` standard library module.

## Required
- `random.random()` → float in [0, 1)
- `random.randint(a, b)` → int in [a, b]
- `random.randrange(start, stop, step)`
- `random.choice(seq)`, `random.choices(population, weights, k)`
- `random.shuffle(list)` (in-place)
- `random.sample(population, k)`
- `random.uniform(a, b)` → float
- `random.gauss(mu, sigma)`, `random.normalvariate(mu, sigma)`
- `random.seed(a=None)`

## Implementation Notes
- Use Rust `rand` crate as backend
- Global RNG state via thread-local
