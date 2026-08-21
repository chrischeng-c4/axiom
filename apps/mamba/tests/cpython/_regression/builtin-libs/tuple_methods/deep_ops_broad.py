# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
# tuple deep ops broad

# construction
t1 = (1, 2, 3)
print(t1)
t2 = tuple()
print(t2)
t3 = tuple([1, 2, 3])
print(t3)
t4 = (42,)  # single-element tuple
print(t4)
print(len(t4))

# indexing
t = (10, 20, 30, 40, 50)
print(t[0])
print(t[-1])
print(t[2])

# forward slicing
print(t[1:4])
print(t[:3])
print(t[3:])
print(t[:])

# membership
print(20 in t)
print(99 in t)
print(99 not in t)

# concatenation
print((1, 2) + (3, 4))
print(() + (1, 2))

# multiplication
print((1, 2) * 3)
print(("a",) * 4)

# len
print(len(()))
print(len((1,)))
print(len((1, 2, 3, 4)))

# iter
for x in (1, 2, 3):
    print(x)

# count
print((1, 2, 3, 2, 2, 4).count(2))
print((1, 2, 3).count(99))

# index
print((10, 20, 30).index(20))
print((10, 20, 30).index(10))

# comparison
print((1, 2) == (1, 2))
print((1, 2) < (1, 3))
print((1, 2) < (2, 0))
print((1, 2, 3) < (1, 2, 3, 4))

# unpacking
a, b, c = (1, 2, 3)
print(a, b, c)
x, *y = (10, 20, 30)
print(x, y)

# tuple from iterable
print(tuple(range(5)))
print(tuple("abc"))

# nested tuple
nt = (1, (2, 3), (4, (5, 6)))
print(nt[0])
print(nt[1])
print(nt[2][1])
print(nt[2][1][0])

# min/max/sum
print(min((3, 1, 4, 1, 5)))
print(max((3, 1, 4, 1, 5)))
print(sum((1, 2, 3, 4, 5)))
