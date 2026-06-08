# Iterable unpacking — star unpacking from lists

# Basic unpacking
a, b, c = [1, 2, 3]
print(a, b, c)

# Starred unpacking: first, *rest
a, *rest, z = [1, 2, 3, 4, 5]
print(a, rest, z)
