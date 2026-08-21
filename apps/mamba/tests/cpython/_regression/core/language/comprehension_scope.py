# Language conformance: comprehension scope isolation and walrus (P0-R5, P0-R6).
# Tests list/dict/set comprehension, generator expressions, scope isolation,
# and walrus operator := (PEP 572).

# List comprehension
squares = [x * x for x in range(5)]
print(squares)

# Dict comprehension
d = {k: k * 2 for k in range(4)}
print(d)

# Set comprehension (sorted for deterministic output)
s = sorted([v % 3 for v in range(9)])
print(s)

# Generator expression with sum
total = sum(n * n for n in range(6))
print(total)

# List comprehension with condition
evens = [n for n in range(10) if n % 2 == 0]
print(evens)

# Nested list comprehension (flatten)
matrix = [[1, 2, 3], [4, 5, 6], [7, 8, 9]]
flat = [cell for row in matrix for cell in row]
print(flat)

# P0-R5: Scope isolation — loop variable must not leak
x = 99
vals = [x * x for x in range(5)]
print(vals)
print(x)

# P0-R5: Scope isolation with dict comprehension
k = "outer"
d2 = {k: k * 10 for k in range(3)}
print(d2)
print(k)

# P0-R6: Walrus operator in list comprehension
results = [y := n * 2 for n in range(4)]
print(results)
print(y)
