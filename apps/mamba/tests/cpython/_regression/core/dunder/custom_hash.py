# Dunder conformance: class with __hash__ defined.
# Extracted from CPython 3.12 test_hash.py FixedHash pattern.
class MyHash:
    def __init__(self, val):
        self.val = val

    def __hash__(self):
        return self.val * 31

a = MyHash(5)
b = MyHash(5)
c = MyHash(3)
print(hash(a))
print(hash(b))
print(hash(a) == hash(b))
print(hash(a) == hash(c))
