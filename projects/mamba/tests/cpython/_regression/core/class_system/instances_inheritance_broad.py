# class operations deep broad

# basic class with init
class Point:
    def __init__(self, x, y):
        self.x = x
        self.y = y
    def total(self):
        return self.x + self.y

p = Point(3, 4)
print(p.x, p.y)
print(p.total())

# class with method calling other methods
class Calc:
    def __init__(self, n):
        self.n = n
    def double(self):
        return self.n * 2
    def quad(self):
        return self.double() * 2

c = Calc(5)
print(c.double())
print(c.quad())

# inheritance
class Animal:
    def __init__(self, name):
        self.name = name
    def greet(self):
        return "I am " + self.name

class Dog(Animal):
    def bark(self):
        return "woof"

dog = Dog("rex")
print(dog.greet())
print(dog.bark())
print(dog.name)

# super() call
class Base:
    def __init__(self, v):
        self.v = v
    def describe(self):
        return "base: " + str(self.v)

class Derived(Base):
    def __init__(self, v, w):
        super().__init__(v)
        self.w = w
    def describe(self):
        return super().describe() + " extra: " + str(self.w)

der = Derived(10, 20)
print(der.v)
print(der.w)
print(der.describe())

# multiple instances
p1 = Point(1, 2)
p2 = Point(10, 20)
print(p1.total(), p2.total())
print(p1.x, p2.x)

# method as attribute
class Adder:
    def __init__(self, base):
        self.base = base
    def add(self, x):
        return self.base + x

a = Adder(100)
print(a.add(1))
print(a.add(99))
print(a.base)

# chained method calls
class Builder:
    def __init__(self):
        self.parts = []
    def add(self, x):
        self.parts.append(x)
        return self
    def build(self):
        return self.parts

b = Builder()
b.add(1).add(2).add(3)
print(b.build())

# class with dunder repr
class Vec:
    def __init__(self, x, y):
        self.x = x
        self.y = y
    def __repr__(self):
        return "Vec(" + str(self.x) + ", " + str(self.y) + ")"

v = Vec(3, 4)
print(repr(v))

# eq method
class Item:
    def __init__(self, id):
        self.id = id
    def __eq__(self, other):
        return self.id == other.id

i1 = Item(1)
i2 = Item(1)
i3 = Item(2)
print(i1 == i2)
print(i1 == i3)
print(i2 == i3)
