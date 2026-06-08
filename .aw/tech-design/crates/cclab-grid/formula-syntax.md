---
title: Grid Formula Syntax Guide
source: SDD improve-grid-maturity proposal
date: 2026-01-28
---

# Formula Syntax Guide

This guide describes the formula syntax and supported functions for cclab-grid.

## Basic Syntax

Formulas always start with an `=` sign.

- **Numbers**: `123`, `12.34`
- **Strings**: `"Hello World"`
- **Booleans**: `TRUE`, `FALSE`
- **Cell References**: `A1`, `B10`
- **Absolute References**: `$A$1`, `A$1`, `$A1`
- **Sheet References**: `'Sheet1'!A1`, `Sales!B2`
- **Ranges**: `A1:B10`, `C:C` (full column), `5:5` (full row)

## Operators

- **Arithmetic**: `+`, `-`, `*`, `/`, `^`, `%`
- **Comparison**: `=`, `<>`, `>`, `<`, `>=`, `<=`
- **Concatenation**: `&`

## Functions

### Mathematical
- `SUM(range)`: Adds all numeric values.
- `AVERAGE(range)`: Returns the average of numeric values.
- `COUNT(range)`: Counts numeric values.
- `COUNTA(range)`: Counts non-empty values.
- `MIN(range)` / `MAX(range)`: Returns smallest/largest value.
- `ABS(number)`: Returns absolute value.
- `ROUND(number, decimals)`: Rounds to specified decimal places.
- `FLOOR(number, significance)` / `CEILING(number, significance)`: Rounds down/up.
- `SQRT(number)`: Returns square root.
- `POWER(base, exp)`: Returns base raised to power.

### Lookup & Reference
- `VLOOKUP(lookup_value, table_array, col_index, [range_lookup])`: Searches first column.
- `HLOOKUP(lookup_value, table_array, row_index, [range_lookup])`: Searches first row.
- `MATCH(lookup_value, lookup_array, [match_type])`: Returns relative position.
- `INDEX(array, row_num, [col_num])`: Returns value at intersection.

### Conditional
- `SUMIF(range, criteria, [sum_range])`: Sums cells matching criteria.
- `COUNTIF(range, criteria)`: Counts cells matching criteria.
- `AVERAGEIF(range, criteria, [average_range])`: Averages cells matching criteria.

## Wildcards
`VLOOKUP` and `MATCH` (exact match) support:
- `*`: Matches any sequence of characters.
- `?`: Matches any single character.

## Array Formulas
Array formulas (Dynamic Arrays) automatically spill results into adjacent cells.
Example: `=FILTER(A1:B10, A1:A10 > 5)` will populate multiple rows.
If the spill range is blocked, `#SPILL!` error is returned.