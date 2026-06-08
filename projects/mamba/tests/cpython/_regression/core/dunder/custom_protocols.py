# class protocol: __len__, __getitem__, __setitem__, __contains__

class Box:
    def __init__(self, items):
        self.items = items
    def __len__(self):
        return len(self.items)
    def __getitem__(self, i):
        return self.items[i]
    def __setitem__(self, i, v):
        self.items[i] = v
    def __contains__(self, v):
        return v in self.items

b = Box([10, 20, 30])
print(len(b))
print(b[1])
b[0] = 99
print(b[0])
print(20 in b)
print(50 in b)

# __call__
class Mul:
    def __init__(self, n):
        self.n = n
    def __call__(self, x):
        return x * self.n

m = Mul(3)
print(m(5))
print(m(10))

# __str__ vs __repr__
class Point:
    def __init__(self, x, y):
        self.x = x
        self.y = y
    def __str__(self):
        return "({}, {})".format(self.x, self.y)
    def __repr__(self):
        return "Point({}, {})".format(self.x, self.y)

p = Point(1, 2)
print(p)
print(str(p))
print(repr(p))

# __bool__
class Box2:
    def __init__(self, v):
        self.v = v
    def __bool__(self):
        return self.v != 0

print(bool(Box2(0)))
print(bool(Box2(5)))
if Box2(0):
    print("truthy0")
else:
    print("falsy0")
if Box2(1):
    print("truthy1")
