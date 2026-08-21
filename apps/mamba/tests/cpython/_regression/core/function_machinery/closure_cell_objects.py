# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
"""Closure cell objects (CPython 3.12 oracle).

A nested function that references an enclosing local captures it in a
cell, reachable via __closure__. Cells expose cell_contents, compare by
their contents, and can be built directly via types.CellType.
"""

import types


def make_cell(value):
    def reader():
        return v
    v = value
    return reader.__closure__[0]


def make_empty_cell(fill=False):
    def reader():
        return v
    # `v` is a local of the enclosing scope (assigned below), so the
    # compiler creates a cell — but when fill is False it stays empty.
    if fill:
        v = 1729
    return reader.__closure__[0]


# A captured local is reachable through cell_contents.
assert make_cell(42).cell_contents == 42

# Cells order and compare by their contents.
assert make_cell(2) < make_cell(3)
assert make_cell(-36) == make_cell(-36.0)
assert make_cell(True) > make_empty_cell()
assert make_empty_cell() < make_cell("saturday")

# Two empty cells compare equal.
assert make_empty_cell() == make_empty_cell()

# types.CellType builds a populated cell directly.
filled = types.CellType(1)
assert filled.cell_contents == 1

# An empty cell raises ValueError when its contents are read.
empty = types.CellType()
try:
    empty.cell_contents
    raise AssertionError("expected ValueError reading an empty cell")
except ValueError:
    pass

print("closure_cell_objects OK")
