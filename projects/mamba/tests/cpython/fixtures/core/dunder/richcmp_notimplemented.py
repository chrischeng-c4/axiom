# Dunder conformance: returning NotImplemented triggers reflected op.
# Extracted from CPython 3.12 test_richcmp.py MiscTest principles.
class A:
    def __init__(self, x):
        self.x = x

    def __eq__(self, other):
        return NotImplemented

class B:
    def __init__(self, x):
        self.x = x

    def __eq__(self, other):
        print('B.__eq__ called')
        return self.x == other.x

a = A(1)
b = B(1)
print(a == b)
print(b == a)
