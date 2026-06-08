# T4.1: super().method() return value propagated
# Conformance test: must produce identical output under CPython 3.12 and Mamba.

class Base:
    def value(self):
        return 42

class Child(Base):
    def value(self):
        v = super().value()
        return v + 1

print(Child().value())  # Expected: 43

# Chain: A -> B -> C
class A:
    def compute(self):
        return 10

class B(A):
    def compute(self):
        return super().compute() + 5

class C(B):
    def compute(self):
        return super().compute() + 3

print(C().compute())  # Expected: 18 (10 + 5 + 3)
