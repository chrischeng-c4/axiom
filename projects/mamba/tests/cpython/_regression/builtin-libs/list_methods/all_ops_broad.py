# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
# list broad ops (beyond existing)

# basic construction
print([1, 2, 3])
print([])
print(list(range(5)))
print(list("abc"))
print(list((1, 2, 3)))

# indexing
a = [10, 20, 30, 40, 50]
print(a[0])
print(a[-1])
print(a[2])
print(a[-3])

# slicing
print(a[1:3])
print(a[:3])
print(a[3:])
print(a[::2])
print(a[::-1])
print(a[-3:])
print(a[:-2])

# mutation
b = [1, 2, 3]
b.append(4)
print(b)
b.extend([5, 6])
print(b)
b.insert(0, 0)
print(b)
b.remove(3)
print(b)
b.pop()
print(b)
b.pop(0)
print(b)
b.reverse()
print(b)
b.sort()
print(b)

# count / index
c = [1, 2, 3, 2, 1, 2, 3]
print(c.count(2))
print(c.count(99))
print(c.index(3))

# arithmetic
print([1, 2] + [3, 4])
print([0] * 5)
print([1, 2] * 3)

# contains
print(3 in [1, 2, 3])
print(99 in [1, 2, 3])
print(99 not in [1, 2, 3])

# nested
m = [[1, 2], [3, 4], [5, 6]]
print(m[0][1])
print(m[2])
total = 0
for row in m:
    for val in row:
        total += val
print(total)

# sorted / reverse-sorted
print(sorted([3, 1, 4, 1, 5, 9, 2, 6]))
print(sorted([3, 1, 4, 1, 5, 9, 2, 6], reverse=True))
print(sorted(["banana", "apple", "cherry"]))

# sort with key
words = ["apple", "fig", "banana"]
print(sorted(words, key=len))

# list from comp with filter
print([x for x in range(10) if x % 3 == 0])

# all / any
print(all([True, True, True]))
print(all([True, False, True]))
print(any([False, False, True]))
print(any([False, False, False]))
print(all([]))
print(any([]))

# max / min
print(max([3, 1, 4, 1, 5, 9, 2, 6]))
print(min([3, 1, 4, 1, 5, 9, 2, 6]))
print(sum([1, 2, 3, 4, 5]))