---
change: mamba-p1-bugfix
group: lambda-limitations
date: 2026-03-28
---

# Requirements

Fix lambda limitations: (1) nested lambdas — lambda inside lambda should capture outer scope, (2) default argument capture — lambda x=val should evaluate val at definition time, (3) unary minus in lambda body — lambda: -x should parse correctly.
