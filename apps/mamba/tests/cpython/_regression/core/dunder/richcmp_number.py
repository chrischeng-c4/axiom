# Dunder conformance: custom Number class with all 6 comparison dunders.
# Extracted from CPython 3.12 test_richcmp.py NumberTest.
class Number:
    def __init__(self, x):
        self.x = x

    def __lt__(self, other):
        return self.x < other.x

    def __le__(self, other):
        return self.x <= other.x

    def __eq__(self, other):
        return self.x == other.x

    def __ne__(self, other):
        return self.x != other.x

    def __gt__(self, other):
        return self.x > other.x

    def __ge__(self, other):
        return self.x >= other.x

a = Number(1)
b = Number(2)
c = Number(1)
print(a < b)
print(a <= b)
print(a == b)
print(a != b)
print(a > b)
print(a >= b)
print(a == c)
print(a != c)
print(a <= c)
print(a >= c)
