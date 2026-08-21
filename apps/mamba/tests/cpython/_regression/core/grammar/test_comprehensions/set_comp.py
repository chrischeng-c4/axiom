# RUN: parse
# CPython 3.12 test_comprehensions: set comprehensions

# Simple
sq = {x**2 for x in range(10)}

# With filter
even_sq = {x**2 for x in range(10) if x % 2 == 0}

# From string
chars = {c for c in "hello world" if c != " "}

# Nested
flat = {x for row in [[1, 2], [2, 3], [3, 4]] for x in row}

# With walrus
seen = {y for x in [1, 2, 2, 3, 3] if (y := x * 10) not in {10}}
