# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
x = [1, 2, 3]
x.append(4)
print(x)
x.extend([5, 6])
print(x)
x.insert(0, 0)
print(x)
v = x.pop()
print(v, x)
x.remove(3)
print(x)