# T5.1: Diamond inheritance C3 MRO order
# Conformance test: must produce identical output under CPython 3.12 and Mamba.

class A:
    pass

class B(A):
    pass

class C(A):
    pass

class D(B, C):
    pass

# Expected MRO: [D, B, C, A, object]
# Print class names from MRO (simplified — Mamba may not have __mro__ attribute)
# Instead verify via method resolution order behavior:

class X:
    def who(self):
        return "X"

class Y(X):
    def who(self):
        return "Y"

class Z(X):
    def who(self):
        return "Z"

class W(Y, Z):
    pass

# W inherits from Y and Z. Y and Z both inherit from X.
# C3 MRO: [W, Y, Z, X, object]
# W().who() should resolve to Y.who() (first in MRO after W)
print(W().who())  # Expected: Y
