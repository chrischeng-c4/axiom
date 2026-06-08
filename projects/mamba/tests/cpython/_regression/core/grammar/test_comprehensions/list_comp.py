# RUN: parse
# CPython 3.12 test_comprehensions: list comprehensions

# Simple
squares = [x**2 for x in range(10)]

# With filter
evens = [x for x in range(20) if x % 2 == 0]

# Nested loops
pairs = [(x, y) for x in range(3) for y in range(3)]

# Nested with filter
filtered = [(x, y) for x in range(5) for y in range(5) if x != y]

# Nested comprehension (list of lists)
matrix = [[i * j for j in range(5)] for i in range(5)]

# With conditional expression
signs = ["pos" if x > 0 else "neg" if x < 0 else "zero" for x in [-1, 0, 1]]

# Walrus in comprehension
results = [y for x in range(10) if (y := x * x) > 10]

# Multiple conditions
multi = [x for x in range(100) if x % 2 == 0 if x % 3 == 0]

# Unpacking in comprehension
data = [(1, 2), (3, 4), (5, 6)]
sums = [a + b for a, b in data]
