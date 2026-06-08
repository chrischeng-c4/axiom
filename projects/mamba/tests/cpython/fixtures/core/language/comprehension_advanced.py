# Advanced comprehensions: nested for, multiple filters, dict/set comp

# Nested for (product)
pairs = [(x, y) for x in range(3) for y in range(3) if x != y]
print(pairs)

# Triple nesting
triples = [(a, b, c) for a in range(2) for b in range(2) for c in range(2)]
print(len(triples))
print(triples[0])
print(triples[-1])

# Filter conditions (combined with and)
result = [x for x in range(20) if x % 2 == 0 and x > 5]
print(result)

# Dict comp
d = {x: x * x for x in range(5)}
print(sorted(d.items()))

# Dict comp with condition
d2 = {x: x * 10 for x in range(10) if x % 3 == 0}
print(sorted(d2.items()))

# Set comp
s = {x % 3 for x in range(10)}
print(sorted(s))

# Nested list comp flattening
matrix = [[1, 2, 3], [4, 5, 6], [7, 8, 9]]
flat = [x for row in matrix for x in row]
print(flat)

# Comprehension with transformation
words = ["hello", "world", "foo"]
caps = [w.upper() for w in words]
print(caps)

# Generator expression (not a list)
g = (x * 2 for x in range(5))
print(sum(g))

# Dict from list of tuples
pairs = [("a", 1), ("b", 2), ("c", 3)]
d3 = {k: v for k, v in pairs}
print(sorted(d3.items()))

# Conditional expression inside
mapped = [x if x > 0 else -x for x in [-2, -1, 0, 1, 2]]
print(mapped)
