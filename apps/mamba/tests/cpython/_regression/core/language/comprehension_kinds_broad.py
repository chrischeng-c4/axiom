# comprehension kinds: list / dict / set / gen broad

# list comp basic
print([x for x in range(5)])
print([x * 2 for x in range(5)])
print([x * x for x in [1, 2, 3, 4]])

# list comp with filter
print([x for x in range(10) if x % 2 == 0])
print([x for x in range(10) if x > 3])
print([x * 10 for x in range(10) if x % 3 == 0])

# list comp over string
print([c.upper() for c in "hello"])
print([len(w) for w in ["one", "two", "three"]])

# dict comp basic
print({x: x * x for x in range(5)})
print({x: str(x) for x in range(3)})

# dict comp with filter
d = {x: x * x for x in range(10) if x % 2 == 0}
print(sorted(d.items()))

# dict comp from list
words = ["apple", "banana", "cherry"]
lens = {w: len(w) for w in words}
print(sorted(lens.items()))

# set comp basic
print(sorted({x for x in range(5)}))
print(sorted({x * 2 for x in range(5)}))
print(sorted({x % 3 for x in range(10)}))

# set comp with filter
s = {x for x in range(20) if x % 4 == 0}
print(sorted(s))

# gen expr in sum
print(sum(x for x in range(10)))
print(sum(x * x for x in range(5)))
print(sum(x for x in range(100) if x % 7 == 0))

# gen expr in max/min
print(max(x * 2 for x in range(10)))
print(min(x + 5 for x in range(10)))

# gen expr in any/all
print(any(x > 5 for x in [1, 2, 3, 10]))
print(any(x > 100 for x in [1, 2, 3, 10]))
print(all(x > 0 for x in [1, 2, 3, 10]))
print(all(x > 2 for x in [1, 2, 3, 10]))

# nested comp
matrix = [[1, 2, 3], [4, 5, 6], [7, 8, 9]]
flat = [x for row in matrix for x in row]
print(flat)

# nested squared
print([[x * y for x in range(3)] for y in range(3)])

# dict with tuple key
pairs = [(1, "a"), (2, "b"), (3, "c")]
d2 = {}
for k, v in pairs:
    d2[k] = v
print(sorted(d2.items()))
