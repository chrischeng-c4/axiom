# Contributing to vat

## Brief

How to change `apps/vat`. What it promises and the work roots it owns live in
[README.md](README.md); repository-wide authoring and verification rules live
in the root [CONTRIBUTING.md](../../CONTRIBUTING.md).

Use `product-deliver` for authorized work. QA owns the red e2e case and its
registration. Dev owns the red unit test and scoped implementation. A fresh
`vat-qa` runs the declared complete gate. The controller owns Git, tracker,
and acceptance. Legacy AW use is explicit-only.

## Verification

| Gate | Command |
|---|---|
| unit + colocated tests | `cargo test -p vat` |
