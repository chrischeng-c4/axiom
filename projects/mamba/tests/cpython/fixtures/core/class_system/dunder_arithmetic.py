# Regression: user-class binary operators must route through __op__/__rop__
# dunders when at least one operand is a class instance. Prior to the fix,
# `a + b` for class instances went straight to mb_add (primitive builtin)
# and returned None because no primitive type matched.

class V:
    def __init__(self, x):
        self.x = x
    def __add__(self, other):
        return V(self.x + other.x)
    def __sub__(self, other):
        return V(self.x - other.x)
    def __mul__(self, other):
        return V(self.x * other.x)
    def __repr__(self):
        return "V(" + str(self.x) + ")"

a = V(10)
b = V(3)
print(a + b)
print(a - b)
print(a * b)

# Chained binary ops
print((a + b) + b)

# Reflected dunder: class-instance on rhs only
class Box:
    def __init__(self, x):
        self.x = x
    def __radd__(self, other):
        return Box(other + self.x)
    def __repr__(self):
        return "Box(" + str(self.x) + ")"

print(5 + Box(7))

# Mixed: int + class with only __radd__
class Accum:
    def __init__(self, x):
        self.x = x
    def __add__(self, other):
        return Accum(self.x + other)
    def __repr__(self):
        return "Accum(" + str(self.x) + ")"

print(Accum(1) + 2)
print(Accum(1) + 2 + 3)
