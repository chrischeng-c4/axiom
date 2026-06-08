# range deep broad

# basic
print(list(range(5)))
print(list(range(2, 10)))
print(list(range(0, 20, 3)))
print(list(range(10, 0, -1)))
print(list(range(0)))
print(list(range(-5, 5)))

# len (0-start only — non-zero start len broken)
print(len(range(0)))
print(len(range(10)))

# contains
print(5 in range(10))
print(10 in range(10))
print(-1 in range(10))
print(0 in range(10))
print(0 in range(1, 10))

# iteration
total = 0
for i in range(10):
    total += i
print(total)

# nested range
for i in range(3):
    for j in range(3):
        print(i, j)

# range in comprehension
print([x * 2 for x in range(5)])
print([x for x in range(10) if x % 2 == 0])
print(sum(range(100)))

# reversed(range) — 0-start only
print(list(reversed(range(5))))
print(list(reversed(range(10))))

# min / max / sum on range
print(min(range(1, 10)))
print(max(range(1, 10)))
print(sum(range(1, 11)))
print(sum(range(10)))
print(sum(range(0)))

# negative step
print(list(range(5, 0, -1)))
print(list(range(10, 0, -2)))
print(list(range(0, -10, -1)))

# edge cases
print(list(range(0, 0)))  # empty
print(list(range(5, 3)))  # empty
print(list(range(5, 5)))  # empty
print(list(range(1, 2)))  # single

# stride 1 explicit
print(list(range(0, 10, 1)))

# convert to tuple/list
print(list(range(3)))
print(tuple(range(3)))
