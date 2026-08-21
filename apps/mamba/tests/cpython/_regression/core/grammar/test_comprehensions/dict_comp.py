# RUN: parse
# CPython 3.12 test_comprehensions: dict comprehensions

# Simple
squares = {x: x**2 for x in range(10)}

# With filter
even_sq = {x: x**2 for x in range(10) if x % 2 == 0}

# Inverted mapping
inverted = {v: k for k, v in {"a": 1, "b": 2}.items()}

# From pairs
pairs = [("a", 1), ("b", 2), ("c", 3)]
d = {k: v for k, v in pairs}

# Nested
nested = {i: {j: i * j for j in range(3)} for i in range(3)}

# With conditional value
classified = {x: ("even" if x % 2 == 0 else "odd") for x in range(10)}
