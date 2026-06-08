# RUN: parse
# CPython 3.12 test_comprehensions: generator expressions

# Simple generator expression
total = sum(x**2 for x in range(10))

# With filter
even_sum = sum(x for x in range(100) if x % 2 == 0)

# Nested
flat_sum = sum(x for row in [[1, 2], [3, 4]] for x in row)

# In function call
result = list(x * 2 for x in range(5))
joined = ",".join(str(x) for x in range(5))

# Multiple generators
all_positive = all(x > 0 for x in [1, 2, 3])
any_negative = any(x < 0 for x in [1, -2, 3])

# With conditional expression
mapped = tuple("even" if x % 2 == 0 else "odd" for x in range(5))
