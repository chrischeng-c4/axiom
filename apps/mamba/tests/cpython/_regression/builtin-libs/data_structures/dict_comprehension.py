# Dict comprehension and merge
d1 = {x: x ** 2 for x in range(5)}
print(d1)
# Dict merge operator (PEP 584)
a = {'x': 1, 'y': 2}
b = {'y': 3, 'z': 4}
c = a | b
print(c)
# Merge overwrites left with right
d = b | a
print(d)
# In-place merge
a |= {'w': 5}
print(a)
# Insertion order
e = {}
e['c'] = 3
e['a'] = 1
e['b'] = 2
print(list(e.keys()))
