"""Behavior contract for language imports.

Each block tests one semantic rule. Expected values are what CPython
3.12 produces. mamba PASS = matches CPython exactly.

# type-regime: monomorphic
"""

import sys
import math

# Rule 1: import binds module to name in current namespace
assert "math" in dir(), "math in namespace after import"

# Rule 2: Modules are cached in sys.modules — import twice gives same object
import math as _math2
assert math is _math2, "re-import gives same module object"

# Rule 3: from...import brings specific names; module not necessarily in namespace
from math import log, log2
assert callable(log), "log callable"
assert abs(log(1.0) - 0.0) < 1e-10, f"log(1) = {log(1.0)!r}"
assert abs(log2(8.0) - 3.0) < 1e-10, f"log2(8) = {log2(8.0)!r}"

# Rule 4: import...as doesn't bind original name
import math as _m
assert callable(_m.sqrt), "math.sqrt via alias"
# _m refers to math module
assert _m is math, "alias is same module"

# Rule 5: ImportError on unknown module
_raised = False
try:
    import _nonexistent_module_xyz_abc  # type: ignore[import-not-found]
except ImportError:
    _raised = True
assert _raised, "unknown module should raise ImportError"

# Rule 6: from...import nonexistent name raises ImportError
_raised2 = False
try:
    from math import nonexistent_name  # type: ignore[attr-defined]
except ImportError:
    _raised2 = True
assert _raised2, "nonexistent attr import should raise ImportError"

# Rule 7: sys.modules is a dict
assert isinstance(sys.modules, dict), f"sys.modules type = {type(sys.modules)!r}"

# Rule 8: Module attributes accessible after import
import os
assert hasattr(os, "getcwd"), "os.getcwd accessible"
assert hasattr(os, "sep"), "os.sep accessible"
assert isinstance(os.sep, str), f"os.sep type = {type(os.sep)!r}"

# Rule 9: Nested module accessible after dotted import
import os.path
assert callable(os.path.abspath), "os.path.abspath accessible"
assert callable(os.path.exists), "os.path.exists accessible"

# Rule 10: importlib.import_module is equivalent to import
import importlib
_os = importlib.import_module("os")
assert _os is os, "importlib returns same cached module"

print("behavior OK")
