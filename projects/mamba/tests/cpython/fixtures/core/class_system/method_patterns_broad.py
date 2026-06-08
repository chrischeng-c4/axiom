# class method patterns broad

# basic instance method
class Counter:
    def __init__(self):
        self.n = 0
    def incr(self):
        self.n += 1
    def get(self):
        return self.n

c = Counter()
c.incr()
c.incr()
c.incr()
print(c.get())

# multiple methods interacting
class Stack:
    def __init__(self):
        self.items = []
    def push(self, x):
        self.items.append(x)
    def pop(self):
        return self.items.pop()
    def size(self):
        return len(self.items)
    def peek(self):
        return self.items[-1]

s = Stack()
s.push(1)
s.push(2)
s.push(3)
print(s.size())
print(s.peek())
print(s.pop())
print(s.size())
print(s.pop())
print(s.pop())
print(s.size())

# method returning another instance
class Num:
    def __init__(self, v):
        self.v = v
    def add(self, n):
        return Num(self.v + n)
    def mul(self, n):
        return Num(self.v * n)
    def show(self):
        return self.v

n = Num(5)
print(n.show())
n2 = n.add(3)
print(n2.show())
n3 = n2.mul(2)
print(n3.show())

# class-level attributes
class Config:
    version = "1.0"
    name = "myapp"

print(Config.version)
print(Config.name)

# class + instance attr
class Circle:
    pi = 3.14159
    def __init__(self, r):
        self.r = r
    def area(self):
        return self.pi * self.r * self.r

c1 = Circle(1)
c2 = Circle(2)
print(c1.area())
print(c2.area())
print(c1.pi)
print(Circle.pi)

# method calling other method
class Calc:
    def __init__(self, start):
        self.n = start
    def double(self):
        self.n = self.n * 2
        return self
    def incr(self):
        self.n += 1
        return self
    def value(self):
        return self.n

ca = Calc(5)
ca.double()
print(ca.value())
ca.incr()
print(ca.value())
ca.double()
print(ca.value())

# chained
cb = Calc(1)
cb.incr()
cb.double()
cb.incr()
print(cb.value())
