class Point:
    def __init__(self, x, y):
        self.x = x
        self.y = y
    def __eq__(self, other):
        return self.x == other.x and self.y == other.y
    def __hash__(self):
        return hash((self.x, self.y))

p1 = Point(1, 2)
p2 = Point(1, 2)
p3 = Point(3, 4)

print(p1 == p2)
print(p1 == p3)
print(hash(p1) == hash(p2))

class Container:
    def __init__(self, items):
        self.items = items
    def __contains__(self, x):
        return x in self.items

ct = Container([1, 2, 3])
print(1 in ct)
print(99 in ct)

class Sized:
    def __init__(self, n):
        self.n = n
    def __len__(self):
        return self.n

sz = Sized(5)
print(len(sz))
print(bool(Sized(0)))
print(bool(Sized(1)))

class Indexed:
    def __init__(self, data):
        self.data = data
    def __getitem__(self, i):
        return self.data[i]

idx = Indexed([10, 20, 30])
print(idx[0])
print(idx[-1])

class Named:
    def __init__(self, name):
        self.name = name
    def __str__(self):
        return self.name

nm = Named("foo")
print(nm)
print(str(nm))

class Counter:
    def __init__(self):
        self.count = 0
    def __call__(self, n):
        self.count += n
        return self.count

cnt = Counter()
print(cnt(1))
print(cnt(5))
print(cnt(10))
print(cnt.count)

class Truth:
    def __init__(self, v):
        self.v = v
    def __bool__(self):
        return self.v

print(bool(Truth(True)))
print(bool(Truth(False)))
