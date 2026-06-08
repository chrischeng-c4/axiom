# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
# issubclass(child, (T1, T2, ...)) — true iff child is a subclass of any
# tuple element. The single-arg form already worked; the tuple form
# previously stringified the tuple and always returned False.

# Built-in hierarchy
print(issubclass(bool, (int,)))            # True (bool ⊆ int)
print(issubclass(bool, (int, float)))      # True
print(issubclass(bool, (str, list)))       # False
print(issubclass(int, (str, bool)))        # False (int is not a subclass of bool)
print(issubclass(int, (object,)))          # True
print(issubclass(str, (int, str, float)))  # True

# User-defined hierarchy
class A: pass
class B(A): pass
class C: pass

print(issubclass(B, (A,)))                 # True
print(issubclass(B, (A, C)))               # True
print(issubclass(B, (C,)))                 # False
print(issubclass(C, (A, B)))               # False
print(issubclass(A, (A,)))                 # True (reflexive)

# Empty tuple → always False
print(issubclass(int, ()))                 # False

# Single-element tuple matches single-class form
print(issubclass(bool, (int,)) == issubclass(bool, int))  # True

# Exception hierarchy
print(issubclass(ValueError, (Exception,)))           # True
print(issubclass(ValueError, (TypeError, Exception))) # True
print(issubclass(KeyError, (ValueError,)))            # False
