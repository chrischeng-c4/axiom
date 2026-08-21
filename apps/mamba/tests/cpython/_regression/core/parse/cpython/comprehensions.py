# RUN: parse
# CPython-derived: list, dict, set comprehensions and generator expressions

# --- list comprehension ---
squares = [x * x for x in range(10)]

# --- list comp with filter ---
evens = [x for x in range(20) if x % 2 == 0]

# --- nested list comp ---
flat = [x for row in matrix for x in row]

# --- dict comprehension ---
d = {k: k * 2 for k in keys}

# --- dict comp with filter ---
d = {k: k for k in items if k > 0}

# --- set comprehension ---
s = {x * x for x in range(10)}

# --- set comp with filter ---
s = {x for x in items if x > 0}

# --- generator expression (parenthesized) ---
g = (x * x for x in range(10))

# --- generator with filter ---
g = (x for x in items if x > 0)

# NOTE: generator as call arg `sum(x for x in items)` is not yet supported
# — requires call parser to detect implicit generator syntax
