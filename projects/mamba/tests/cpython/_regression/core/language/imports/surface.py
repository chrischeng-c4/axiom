"""Surface contract for language imports.

# type-regime: monomorphic

Probes: import, from...import, import...as, from...import...as,
__name__, __file__, sys.modules caching, importlib.
CPython 3.12 is the oracle.
"""

import sys
import math
import os.path
from math import sqrt, pi
from math import floor as _floor
import importlib

# Basic import
assert math is not None, "import math failed"
assert callable(math.sqrt), "math.sqrt not callable"

# from...import brings name into scope
assert callable(sqrt), "from math import sqrt"
assert abs(sqrt(4.0) - 2.0) < 1e-10, f"sqrt(4) = {sqrt(4.0)!r}"

# pi is a float constant
assert isinstance(pi, float), f"pi type = {type(pi)!r}"
assert abs(pi - 3.141592653589793) < 1e-10, f"pi = {pi!r}"

# import...as aliasing
assert callable(_floor), "import as works"
assert _floor(2.9) == 2, f"floor(2.9) = {_floor(2.9)!r}"

# os.path is accessible via dotted import
assert callable(os.path.join), "os.path.join accessible"
_joined = os.path.join("a", "b", "c")
assert "b" in _joined, f"join = {_joined!r}"

# sys.modules caches imports
assert "math" in sys.modules, "math in sys.modules"
assert sys.modules["math"] is math, "sys.modules[math] is same object"

# importlib.import_module works like import
_math2 = importlib.import_module("math")
assert _math2 is math, "importlib returns cached module"

# __name__ at module level is __main__ (when run directly) or the module name
assert isinstance(__name__, str), f"__name__ type = {type(__name__)!r}"

# Re-import is idempotent (same object)
import math as _math3
assert _math3 is math, "re-import returns same object"

# from...import multiple names
from os.path import basename, dirname
assert callable(basename), "basename accessible"
assert callable(dirname), "dirname accessible"

print("surface OK")
