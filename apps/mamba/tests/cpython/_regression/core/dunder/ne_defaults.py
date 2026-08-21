# Dunder conformance: __ne__ defaults to not __eq__.
# Extracted from CPython 3.12 test_compare.py test_ne_defaults_to_not_eq.
class Cmp:
    def __init__(self, arg):
        self.arg = arg

    def __eq__(self, other):
        return self.arg == other.arg

a = Cmp(1)
b = Cmp(1)
c = Cmp(2)
print(a == b)
print(a != b)
print(a != c)
