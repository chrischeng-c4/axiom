# augmented assignment broad

# int
x = 10
x += 5
print(x)
x -= 3
print(x)
x *= 2
print(x)
x //= 4
print(x)
x **= 2
print(x)
x %= 3
print(x)

# with expressions
y = 100
y += 2 * 3
print(y)
y -= (10 + 5)
print(y)

# float
f = 1.0
f += 0.5
print(f)
f *= 2
print(f)
f /= 4
print(f)

# string
s = "abc"
s += "def"
print(s)
s *= 2
print(s)

# list
lst = [1, 2, 3]
lst += [4, 5]
print(lst)
lst *= 2
print(lst)

# list extend
other = [10, 20]
lst2 = [1]
lst2 += other
print(lst2)

# bitwise int
bits = 0
bits |= 1
bits |= 2
bits |= 4
print(bits)
bits &= 5
print(bits)
bits ^= 3
print(bits)
bits <<= 2
print(bits)
bits >>= 1
print(bits)

# on class attribute
class Counter:
    def __init__(self):
        self.n = 0

c = Counter()
c.n += 1
c.n += 10
print(c.n)

# on dict value
d = {"a": 1}
d["a"] += 10
print(d["a"])

# on list element
lst3 = [1, 2, 3]
lst3[0] += 100
print(lst3)
