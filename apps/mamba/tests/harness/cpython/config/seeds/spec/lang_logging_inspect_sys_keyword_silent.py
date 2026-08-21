# Operational AssertionPass divergence-spec fixture for the silent
# value-contract divergences of `hasattr(logging, 'NOTSET')` (the
# documented "logging exposes the NOTSET=0 level sentinel" — mamba
# returns False), `hasattr(logging, 'Logger')` (the documented
# "logging exposes the Logger class" — mamba returns False),
# `hasattr(logging, 'Formatter')` (the documented "logging exposes
# the Formatter class" — mamba returns False), `hasattr(logging,
# 'getLevelName')` (the documented "logging exposes the
# getLevelName(level) -> str helper" — mamba returns False),
# `hasattr(inspect, 'Parameter')` (the documented "inspect exposes
# the Parameter class" — mamba returns False), `hasattr(inspect,
# 'Signature')` (the documented "inspect exposes the Signature
# class" — mamba returns False), `hasattr(inspect, 'currentframe')`
# (the documented "inspect exposes currentframe()" — mamba returns
# False), `inspect.isclass(int)` (the documented "isclass(int)
# returns True — int is a class" — mamba returns False), `str
# (inspect.signature(fn))` (the documented "signature() returns a
# Signature whose str repr is '(a, b=2)' for fn(a, b=2)" — mamba
# returns '()' — empty parameter list), and `sys.version_info >=
# (3, 0)` (the documented "sys.version_info compares >= a tuple
# of ints — returns True for any Python 3" — mamba returns False
# — version_info comparison does not match).
# Ten-pack pinned to atomic 273.
#
# Behavioral edges that CONFORM on mamba (logging — hasattr DEBUG/
# INFO/WARNING/ERROR/CRITICAL/getLogger/basicConfig + DEBUG==10/
# INFO==20/WARNING==30/ERROR==40/CRITICAL==50. inspect — hasattr
# signature/getmembers/isclass/isfunction/ismethod + getargspec
# removed in 3.11 (both False) + isclass(5)==False + isfunction
# (int)==False + isfunction(lambda)==True. sys — hasattr argv/path/
# modules/platform/version/version_info/exit/getrecursionlimit/
# maxsize/stdout/stderr/stdin/byteorder/getsizeof/settrace + argv/
# path are list, platform/version are str, maxsize>0, byteorder==
# 'little'. keyword — hasattr iskeyword/kwlist/softkwlist +
# iskeyword if/else/for/while/None/True True + iskeyword foo/bar
# False + kwlist is list + 'if' in kwlist + len(kwlist)==35) are
# covered in the matching pass fixture
# `test_logging_inspect_sys_keyword_value_ops`.
import logging
import inspect
import sys


def _myfn(a, b=2):
    return a + b


_ledger: list[int] = []

# 1) hasattr(logging, 'NOTSET') — level sentinel
#    (mamba: returns False)
assert hasattr(logging, "NOTSET") == True; _ledger.append(1)

# 2) hasattr(logging, 'Logger') — Logger class
#    (mamba: returns False)
assert hasattr(logging, "Logger") == True; _ledger.append(1)

# 3) hasattr(logging, 'Formatter') — Formatter class
#    (mamba: returns False)
assert hasattr(logging, "Formatter") == True; _ledger.append(1)

# 4) hasattr(logging, 'getLevelName') — level-name helper
#    (mamba: returns False)
assert hasattr(logging, "getLevelName") == True; _ledger.append(1)

# 5) hasattr(inspect, 'Parameter') — Parameter class
#    (mamba: returns False)
assert hasattr(inspect, "Parameter") == True; _ledger.append(1)

# 6) hasattr(inspect, 'Signature') — Signature class
#    (mamba: returns False)
assert hasattr(inspect, "Signature") == True; _ledger.append(1)

# 7) hasattr(inspect, 'currentframe') — currentframe helper
#    (mamba: returns False)
assert hasattr(inspect, "currentframe") == True; _ledger.append(1)

# 8) inspect.isclass(int) — int is a class
#    (mamba: returns False)
assert inspect.isclass(int) == True; _ledger.append(1)

# 9) str(inspect.signature(_myfn)) — parameter-list repr
#    (mamba: returns '()' — empty signature)
assert str(inspect.signature(_myfn)) == "(a, b=2)"; _ledger.append(1)

# 10) sys.version_info >= (3, 0) — version-tuple comparison
#     (mamba: returns False)
assert (sys.version_info >= (3, 0)) == True; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_logging_inspect_sys_keyword_silent {sum(_ledger)} asserts")
