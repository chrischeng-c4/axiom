class A:
    def __init__(self, x):
        self.x = x
    def hello(self):
        return "A(" + str(self.x) + ")"

class B(A):
    def __init__(self, x, y):
        super().__init__(x)
        self.y = y
    def hello(self):
        return super().hello() + "+B(" + str(self.y) + ")"

class C(B):
    def __init__(self, x, y, z):
        super().__init__(x, y)
        self.z = z
    def hello(self):
        return super().hello() + "+C(" + str(self.z) + ")"

c = C(1, 2, 3)
print(c.hello())
print(c.x, c.y, c.z)

# super() with positional args
class D:
    def sum3(self, a, b, c):
        return a + b + c

class E(D):
    def sum3(self, a, b, c):
        return super().sum3(a, b, c) * 2

print(E().sum3(1, 2, 3))

# diamond
class X:
    def name(self):
        return "X"

class Y(X):
    def name(self):
        return super().name() + "->Y"

class Z(X):
    def name(self):
        return super().name() + "->Z"

class W(Y, Z):
    def name(self):
        return super().name() + "->W"

print(W().name())
print(W.__mro__[0].__name__, W.__mro__[1].__name__, W.__mro__[-1].__name__)
