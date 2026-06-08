# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
x = [1, 2, 3]
y = x.copy()
y.append(4)
print(x)
print(y)
x.clear()
print(x)
print(len(x))