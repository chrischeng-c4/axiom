# __slots__: restricts instance attributes

class Point:
    __slots__ = ('x', 'y')

    def __init__(self, x, y):
        self.x = x
        self.y = y

p = Point(1, 2)
print(p.x)
print(p.y)

# Modify allowed slots
p.x = 10
print(p.x)

# Try to set disallowed attribute
try:
    p.z = 3
except AttributeError:
    print("AttributeError: cannot set z")
