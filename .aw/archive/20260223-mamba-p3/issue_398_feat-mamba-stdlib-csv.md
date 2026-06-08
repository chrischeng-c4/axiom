---
number: 398
title: "feat(mamba): stdlib csv"
state: open
labels: [enhancement, crate:mamba, P3]
---

# #398 — feat(mamba): stdlib csv

## Summary
Implement `csv` module for CSV file reading/writing.

## Required
- `csv.reader(csvfile, delimiter=',')` → row iterator
- `csv.writer(csvfile, delimiter=',')` → writer with `.writerow()`, `.writerows()`
- `csv.DictReader(csvfile)` → dict-per-row iterator
- `csv.DictWriter(csvfile, fieldnames)` → dict writer

## Implementation Notes
- Depends on file I/O (#379)
- Can use simple Rust string splitting; consider `csv` crate for edge cases
