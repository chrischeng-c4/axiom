# Dunder conformance: equal objects must have equal hashes.
# Extracted from CPython 3.12 test_hash.py test_numeric_literals.
print(hash(1) == hash(1.0))
print(hash(0) == hash(0.0))
print(hash(2) == hash(2.0))
