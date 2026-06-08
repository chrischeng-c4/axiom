# copy via builtins broad

# list[:] shallow copy
a = [1, 2, 3]
b = a[:]
print(a)
print(b)
print(a is b)
b.append(99)
print(a)
print(b)

# list(list) shallow copy
c = [1, 2, 3]
d = list(c)
d.append(99)
print(c)
print(d)

# dict.copy
d1 = {"a": 1, "b": 2}
d2 = d1.copy()
print(sorted(d1.items()))
print(sorted(d2.items()))
d2["c"] = 3
print(sorted(d1.items()))
print(sorted(d2.items()))

# set.copy
s1 = {1, 2, 3}
s2 = s1.copy()
print(sorted(s1))
print(sorted(s2))
s2.add(99)
print(sorted(s1))
print(sorted(s2))

# set(set)
s3 = {1, 2}
s4 = set(s3)
s4.add(99)
print(sorted(s3))
print(sorted(s4))

# tuple[:] — tuples are immutable, returns same
t = (1, 2, 3)
t2 = t[:]
print(t == t2)

# slice preserves outer list not inner
nested = [[1, 2], [3, 4]]
nc = nested[:]
# same inner lists
nc[0].append(99)
print(nested)  # also changed
print(nc)

# str[:]
s = "hello"
print(s[:])

# tuple constructor copy
t3 = tuple([1, 2, 3])
print(t3)
t4 = tuple((4, 5, 6))
print(t4)

# list constructor copy
l3 = list([1, 2, 3])
print(l3)
l4 = list(range(5))
print(l4)
