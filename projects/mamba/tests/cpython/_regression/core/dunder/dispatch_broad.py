# dunder comparison
class Box:
    def __init__(self, v):
        self.v = v
    def __lt__(self, other):
        return self.v < other.v
    def __le__(self, other):
        return self.v <= other.v
    def __gt__(self, other):
        return self.v > other.v
    def __ge__(self, other):
        return self.v >= other.v
    def __eq__(self, other):
        return self.v == other.v

b1 = Box(1)
b2 = Box(2)
b3 = Box(1)
print(b1 < b2)
print(b1 <= b3)
print(b2 > b1)
print(b2 >= b1)
print(b1 == b3)
print(b1 < b3)

# dunder len / bool
class Container:
    def __init__(self, items):
        self.items = items
    def __len__(self):
        return len(self.items)
    def __bool__(self):
        return len(self.items) > 0

c = Container([1, 2, 3])
print(len(c))
print(bool(c))

c2 = Container([])
print(len(c2))
print(bool(c2))

# dunder str
class Pair:
    def __init__(self, a, b):
        self.a = a
        self.b = b
    def __str__(self):
        return "<" + str(self.a) + ", " + str(self.b) + ">"

p = Pair(1, 2)
print(str(p))

# dunder getitem
class Seq:
    def __init__(self, data):
        self.data = data
    def __getitem__(self, idx):
        return self.data[idx]
    def __len__(self):
        return len(self.data)

sq = Seq([10, 20, 30, 40])
print(sq[0])
print(sq[2])
print(sq[-1])
print(len(sq))

# dunder contains
class HasItems:
    def __init__(self, vals):
        self.vals = vals
    def __contains__(self, item):
        return item in self.vals

h = HasItems([1, 2, 3])
print(1 in h)
print(99 in h)
