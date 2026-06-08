# Dunder conformance: class with __eq__ defined — == and != dispatch.
# Extracted from CPython 3.12 test_compare.py ComparisonSimpleTest.Cmp.
class MyEq:
    def __init__(self, val):
        self.val = val

    def __eq__(self, other):
        return self.val == other.val

a = MyEq(1)
b = MyEq(1)
c = MyEq(2)
print(a == b)
print(a == c)
print(a != b)
print(a != c)
