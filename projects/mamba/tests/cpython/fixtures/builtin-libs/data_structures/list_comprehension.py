# List comprehension edge cases
print([x * 2 for x in range(5)])
print([x for x in range(10) if x % 2 == 0])
print([x * y for x in range(3) for y in range(3)])
# Nested
print([[j for j in range(3)] for i in range(3)])
# Empty
print([x for x in range(0)])
# With expression
print([i ** 2 for i in range(6)])
