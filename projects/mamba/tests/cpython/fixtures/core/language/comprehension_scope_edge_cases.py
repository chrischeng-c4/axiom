# Comprehension scope edge cases

# PEP 709: list comp variable does NOT leak
x = 'outer'
result = [x for x in range(3)]
print(x)
print(result)

# Nested comprehension scoping
outer = 10
matrix = [[i + j for j in range(3)] for i in range(3)]
print(matrix)
print(outer)

# Walrus operator in comprehension (PEP 572)
results = [y := x**2 for x in range(4)]
print(results, y)

# Set comprehension scope
z = 'kept'
s = {x * 2 for x in range(4)}
print(z)
print(sorted(s))
