# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
# callable() — recognise the full set of Python-callable values:
# functions, lambdas, builtin type constructors, user-defined classes,
# instances with __call__. Mamba previously returned False for all but
# `def`-defined functions because mb_callable only checked TAG_FUNC.

# Functions
def foo(): return 1
print(callable(foo))                  # True

# Lambdas — bound to a name and inline
f = lambda x: x
print(callable(f))                    # True
print(callable(lambda x: x + 1))      # True

# Builtin type constructors (resolve as bare strings)
print(callable(int))                  # True
print(callable(str))                  # True
print(callable(list))                 # True
print(callable(dict))                 # True
print(callable(tuple))                # True
print(callable(set))                  # True

# Builtin functions
print(callable(print))                # True
print(callable(len))                  # True
print(callable(sorted))               # True

# User-defined classes
class C:
    pass
print(callable(C))                    # True

class D:
    def __call__(self): return 1
d = D()
print(callable(d))                    # True
print(callable(D))                    # True

# Bound methods
class E:
    def m(self): return 1
e = E()
print(callable(e.m))                  # True
print(callable(E.m))                  # True

# Plain instances with no __call__
class F: pass
print(callable(F()))                  # False

# Non-callable primitives
print(callable(42))                   # False
print(callable(3.14))                 # False
print(callable("hello"))              # False
print(callable([1, 2, 3]))            # False
print(callable({1: 2}))               # False
print(callable(None))                 # False
print(callable(True))                 # False
