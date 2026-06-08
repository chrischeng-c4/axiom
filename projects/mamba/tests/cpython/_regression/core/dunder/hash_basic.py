# Dunder conformance: hash() of basic types.
# Extracted from CPython 3.12 test_hash.py HashEqualityTestCase.
print(hash(0))
print(hash(1))
print(hash(-1))
print(hash(True))
print(hash(False))
print(type(hash('hello')))
