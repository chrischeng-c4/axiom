# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
# tuple access patterns broad

# index
t = (10, 20, 30, 40, 50)
print(t[0])
print(t[1])
print(t[4])
print(t[-1])
print(t[-2])

# len
print(len(t))
print(len(()))
print(len((1,)))
print(len((1, 2, 3)))

# in / not in
print(10 in t)
print(99 in t)
print(99 not in t)
print(10 not in t)

# iteration
for x in t:
    print(x)

# count
t2 = (1, 2, 3, 2, 1, 2, 4)
print(t2.count(1))
print(t2.count(2))
print(t2.count(99))
print(t2.count(4))

# index method
print(t.index(10))
print(t.index(30))
print(t.index(50))

# tuple + tuple
a = (1, 2)
b = (3, 4)
print(a + b)
print(b + a)
print(a + ())
print(() + b)

# tuple * n
print((1, 2) * 3)
print((0,) * 5)
print(("x",) * 4)

# slice
print(t[1:3])
print(t[:2])
print(t[3:])
print(t[:])
print(t[::2])

# equality
print((1, 2, 3) == (1, 2, 3))
print((1, 2) == (1, 2, 3))
print((1, 2, 3) == (3, 2, 1))

# comparison
print((1, 2, 3) < (1, 2, 4))
print((1, 2) < (1, 2, 3))
print((2,) > (1, 9, 9))

# bool
print(bool((1, 2)))
print(bool(()))
print(bool((0,)))

# nested access
nested = ((1, 2), (3, 4), (5, 6))
print(nested[0])
print(nested[1])
print(nested[0][0])
print(nested[1][1])
print(nested[2][0])

# sum of tuple
print(sum((1, 2, 3, 4, 5)))
print(sum((10, 20, 30)))

# min/max
print(min((3, 1, 2)))
print(max((3, 1, 2)))

# to list
print(list((1, 2, 3)))
print(list(()))
