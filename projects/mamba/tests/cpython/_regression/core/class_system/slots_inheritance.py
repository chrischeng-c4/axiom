# __slots__ across inheritance: subclass slots merge with parent slots

class Base:
    __slots__ = ("x",)
    def __init__(self, x):
        self.x = x

class Child(Base):
    __slots__ = ("y",)
    def __init__(self, x, y):
        super().__init__(x)
        self.y = y

c = Child(1, 2)
print(c.x, c.y)

# Parent slot is inherited — still accessible
b = Base(10)
print(b.x)

# Assigning unlisted slot attribute raises AttributeError
try:
    c.z = 99
    print("no error (unexpected)")
except AttributeError:
    print("AttributeError")
