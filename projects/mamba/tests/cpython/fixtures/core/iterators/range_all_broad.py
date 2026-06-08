# range broad

# basic
print(list(range(5)))
print(list(range(2, 5)))
print(list(range(0, 10, 2)))
print(list(range(10, 0, -1)))
print(list(range(10, 0, -2)))

# empty ranges
print(list(range(0)))
print(list(range(5, 5)))
print(list(range(5, 0)))
print(list(range(0, 5, -1)))

# large range — just check len/first/last
r = range(1000)
print(len(r))
print(r[0])
print(r[-1])
print(r[500])

# negative range
print(list(range(-5, 5)))
print(list(range(-5, 5, 2)))

# membership
print(3 in range(10))
print(10 in range(10))

# sum / min / max of range
print(sum(range(10)))
print(min(range(10)))
print(max(range(10)))

# range iteration
total = 0
for i in range(100):
    total += i
print(total)

# nested range
cells = []
for i in range(3):
    for j in range(3):
        cells.append((i, j))
print(cells)
