# __slots__ broad

class Point:
    __slots__ = ("x", "y")
    def __init__(self, x, y):
        self.x = x
        self.y = y

p = Point(3, 4)
print(p.x)
print(p.y)

# allowed slot access
p.x = 10
p.y = 20
print(p.x)
print(p.y)

# denied: unknown attribute
try:
    p.z = 100
except AttributeError:
    print("denied z")

# slots with methods (methods live on class, not instance)
class Vec:
    __slots__ = ("dx", "dy")
    def __init__(self, dx, dy):
        self.dx = dx
        self.dy = dy
    def length_sq(self):
        return self.dx * self.dx + self.dy * self.dy

v = Vec(3, 4)
print(v.length_sq())

# denied attribute on Vec
try:
    v.name = "foo"
except AttributeError:
    print("denied name")

# multiple instances are separate
v2 = Vec(5, 12)
print(v2.length_sq())
print(v.length_sq())

# read slot after set
v.dx = 100
print(v.dx)
print(v.dy)
