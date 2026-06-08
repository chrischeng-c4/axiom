# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
# list.copy, list slice copy, list operators
a = [1, 2, 3]
b = a.copy()
b.append(4)
print(a)
print(b)

# copy via slice
c = a[:]
c.append(9)
print(a)
print(c)

# list *
print([0] * 5)
print([1, 2] * 3)

# list addition
print([1] + [2, 3])

# list unpacking
a, b, c = [1, 2, 3]
print(a, b, c)

# starred
a, *rest = [1, 2, 3, 4]
print(a, rest)
*head, tail = [1, 2, 3, 4]
print(head, tail)
a, *mid, z = [1, 2, 3, 4, 5]
print(a, mid, z)

# for with tuple unpacking
for a, b in [(1, 2), (3, 4), (5, 6)]:
    print(a + b)

# list.index / count
l = [1, 2, 3, 2, 1]
print(l.index(2))
print(l.count(2))
print(l.count(99))

# list.remove
l = [1, 2, 3, 2, 1]
l.remove(2)
print(l)

# list.reverse (in-place)
l = [1, 2, 3]
l.reverse()
print(l)

# list.sort
l = [3, 1, 4, 1, 5, 9, 2]
l.sort()
print(l)

# list.sort with key
words = ["apple", "pi", "banana"]
words.sort(key=len)
print(words)

# list.insert
l = [1, 2, 4]
l.insert(2, 3)
print(words)
l.insert(0, 0)
print(l)