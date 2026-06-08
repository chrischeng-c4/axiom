# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
t = (1, 2, 3)
print(t)
print(len(t))
print(t[0], t[-1])

# empty and singleton
e = ()
s = (1,)
print(e)
print(s)
print(len(e), len(s))

# tuple from iterable
print(tuple([1, 2, 3]))
print(tuple("abc"))
print(tuple(range(4)))

# indexing and slicing
t5 = (10, 20, 30, 40, 50)
print(t5[2])
print(t5[-1])
print(t5[1:4])
print(t5[::2])

# concatenation / repetition
print((1, 2) + (3, 4))
print((0,) * 5)

# membership
print(2 in (1, 2, 3))
print(99 in (1, 2, 3))

# methods
t6 = (1, 2, 2, 3, 2, 4)
print(t6.count(2))
print(t6.index(3))

# iteration
total = 0
for x in (1, 2, 3, 4, 5):
    total += x
print(total)

# unpacking
a, b, c = (10, 20, 30)
print(a, b, c)

a, *rest = (1, 2, 3, 4, 5)
print(a, rest)

*head, last = (1, 2, 3, 4, 5)
print(head, last)

# comparison
print((1, 2, 3) == (1, 2, 3))
print((1, 2, 3) < (1, 2, 4))
print((1, 2) < (1, 2, 0))

# nested
nt = ((1, 2), (3, 4), (5, 6))
print(nt)
print(nt[1][0])
