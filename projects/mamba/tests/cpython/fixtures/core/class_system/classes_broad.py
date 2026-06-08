# class features

# basic init / method
class Point:
    def __init__(self, x, y):
        self.x = x
        self.y = y

    def dist(self):
        return (self.x * self.x + self.y * self.y) ** 0.5

    def shift(self, dx, dy):
        return Point(self.x + dx, self.y + dy)

p = Point(3, 4)
print(p.x, p.y)
print(p.dist())
q = p.shift(1, 1)
print(q.x, q.y)

# class attributes vs instance attributes
class Counter:
    total = 0
    def __init__(self):
        Counter.total += 1

a = Counter()
b = Counter()
c = Counter()
print(Counter.total)

# method returning self
class Builder:
    def __init__(self):
        self.parts = []
    def add(self, x):
        self.parts.append(x)
        return self
    def build(self):
        return "-".join(self.parts)

b = Builder().add("a").add("b").add("c")
print(b.build())

# inheritance
class Animal:
    def __init__(self, name):
        self.name = name
    def speak(self):
        return "..."
    def greet(self):
        return f"{self.name}: {self.speak()}"

class Dog(Animal):
    def speak(self):
        return "woof"

class Cat(Animal):
    def speak(self):
        return "meow"

dog = Dog("Rex")
cat = Cat("Whisk")
print(dog.greet())
print(cat.greet())

# super
class Base:
    def __init__(self, x):
        self.x = x
    def desc(self):
        return f"x={self.x}"

class Sub(Base):
    def __init__(self, x, y):
        super().__init__(x)
        self.y = y
    def desc(self):
        return f"{super().desc()}, y={self.y}"

s = Sub(10, 20)
print(s.desc())

# multiple inheritance
class A:
    def who(self):
        return "A"

class B:
    def who(self):
        return "B"

class AB(A, B):
    pass

ab = AB()
print(ab.who())  # A (MRO left-first)

# attr exists
class Holder:
    def __init__(self):
        self.val = 10

h = Holder()
print(hasattr(h, "val"))
print(hasattr(h, "missing"))
print(getattr(h, "val"))
print(getattr(h, "missing", "default"))
setattr(h, "extra", 42)
print(h.extra)
