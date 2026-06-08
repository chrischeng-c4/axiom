# tuple unpacking
x, y = 1, 2
print(x, y)

a, b, c = "xyz"
print(a, b, c)

# swap
a, b = 1, 2
a, b = b, a
print(a, b)

# from list
x, y, z = [10, 20, 30]
print(x, y, z)

# from tuple
x, y, z = (100, 200, 300)
print(x, y, z)

# starred
a, *rest = [1, 2, 3, 4, 5]
print(a, rest)

*init, last = [1, 2, 3, 4, 5]
print(init, last)

first, *middle, last = [1, 2, 3, 4, 5, 6]
print(first, middle, last)

# nested
(a, b), (c, d) = (1, 2), (3, 4)
print(a, b, c, d)

# nested tuple loop
data = [(1, "a"), (2, "b"), (3, "c")]
for n, s in data:
    print(n, s)

# augmented assignment (fresh name)
counter = 10
counter += 5
print(counter)
counter -= 3
print(counter)
counter *= 2
print(counter)
counter //= 3
print(counter)
counter **= 2
print(counter)

# in-place list / string
lst = [1, 2, 3]
lst += [4, 5]
print(lst)

msg = "hello"
msg += " world"
print(msg)
