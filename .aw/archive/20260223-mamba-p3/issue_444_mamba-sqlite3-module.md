---
number: 444
title: "mamba: sqlite3 module"
state: open
labels: [enhancement, crate:mamba, P3]
---

# #444 — mamba: sqlite3 module

## Description

Implement `sqlite3` module — Python's built-in database interface.

## Requirements

- R1: `sqlite3.connect(database)` — open/create database
- R2: `connection.cursor()` — create cursor object
- R3: `cursor.execute(sql, parameters=())` — execute SQL with parameterized queries
- R4: `cursor.executemany(sql, seq_of_params)` — execute for multiple parameter sets
- R5: `cursor.fetchone()`, `fetchall()`, `fetchmany(size)`
- R6: `connection.commit()`, `connection.rollback()`
- R7: `connection.close()`
- R8: Context manager support: `with sqlite3.connect(db) as conn:`
- R9: `cursor.description` — column metadata
- R10: Row factory: `connection.row_factory = sqlite3.Row`

## Priority

P3 — important for data-heavy applications.
