
# List mutation and query methods
a = [1, 2, 3]
a.append(4)
print(a)
a.extend([5, 6])
print(a)
a.insert(0, 0)
print(a)
x = a.pop()
print(x)
print(a)
a.remove(3)
print(a)
a.reverse()
print(a)
b = a.copy()
print(b)
a.clear()
print(a)
print(b)
# index and count
c = [1, 2, 3, 2, 1]
print(c.index(2))
print(c.count(2))
