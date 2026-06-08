"""Behavior contract for builtins.callable.

Each block tests one semantic rule. Expected values are what CPython
3.12 produces. mamba PASS = matches CPython exactly.

# type-regime: monomorphic
"""

# Rule 1: built-in functions
assert callable(len) is True
assert callable(print) is True
assert callable(range) is True
assert callable(type) is True
assert callable(object) is True

# Rule 2: user-defined functions
def _fn():
    return 1
assert callable(_fn) is True, "user fn not callable"

# Rule 3: lambda
_lam = lambda x: x
assert callable(_lam) is True, "lambda not callable"

# Rule 4: classes are callable
assert callable(int) is True
assert callable(str) is True
assert callable(list) is True

# Rule 5: instances with __call__
class _HasCall:
    def __call__(self):
        return 42
assert callable(_HasCall()) is True, "instance with __call__ not callable"

# Rule 6: instances without __call__
class _NoCall:
    pass
assert callable(_NoCall()) is False, "instance without __call__ is callable"

# Rule 7: scalars and containers are not callable
assert callable(42) is False
assert callable(3.14) is False
assert callable("hello") is False
assert callable(b"bytes") is False
assert callable([]) is False
assert callable({}) is False
assert callable(()) is False
assert callable(None) is False
assert callable(True) is False

# Rule 8: callable wraps __call__ check
class _DynCall:
    pass
obj = _DynCall()
assert callable(obj) is False
obj.__call__ = lambda: 1
# Note: dynamically added __call__ to instance (not class) is NOT callable per CPython
assert callable(obj) is False, "instance __call__ on instance dict not callable per CPython"

print("behavior OK")
