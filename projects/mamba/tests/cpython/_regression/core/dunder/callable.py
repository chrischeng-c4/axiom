# Dunder conformance: __call__ on custom class.
class Adder:
    def __init__(self, n):
        self.n = n

    def __call__(self, x):
        return self.n + x

add5 = Adder(5)
print(add5(3))
print(add5(10))
print(add5(0))

class Multiplier:
    def __init__(self, factor):
        self.factor = factor

    def __call__(self, a, b):
        return a * b * self.factor

m = Multiplier(2)
print(m(3, 4))
