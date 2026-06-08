# Operational AssertionPass divergence-spec fixture for the silent
# value-contract divergences of `sys.platform` (the documented "on
# macOS sys.platform is 'darwin'" — mamba returns 'macos'),
# `sys.version_info[0]` (the documented "version_info is integer-
# indexable; [0] is the major version" — mamba raises KeyError '0'),
# `type(sys.float_info).__name__` (the documented "sys.float_info is
# a named-tuple struct sequence reporting type name 'float_info'" —
# mamba returns 'dict'), `type(os.environ).__name__` (the documented
# "os.environ is an instance of os._Environ" — mamba returns 'dict'),
# `'HOME' in os.environ` (the documented "the HOME environment
# variable is exposed as a key" — mamba returns False — the environ
# dict is empty on probe), `hasattr(os.path, 'isabs')` (the
# documented "os.path exposes isabs()" — mamba returns False),
# `platform.system()` (the documented "platform.system() returns
# 'Darwin' on macOS" — mamba returns 'macos'), `platform.machine()`
# (the documented "platform.machine() returns 'arm64' on Apple
# Silicon" — mamba returns 'aarch64'), `hasattr(platform,
# 'python_implementation')` (the documented "platform exposes
# python_implementation()" — mamba returns False), and
# `hasattr(gc, 'garbage')` (the documented "gc exposes the garbage
# list of uncollectable objects" — mamba returns False).
# Ten-pack pinned to atomic 261.
#
# Behavioral edges that CONFORM on mamba (sys — hasattr surface
# version/version_info/platform/argv/modules/path/stdin/stdout/
# stderr/exit/maxsize/executable/getsizeof/getrecursionlimit/
# setrecursionlimit/byteorder/hexversion/intern + types/values for
# argv/modules/path/executable, maxsize>0, hexversion>0, byteorder
# =='little', version_info.major==3 / .minor==12, sys.intern
# roundtrip, getsizeof returns positive. os — hasattr surface +
# os.sep / linesep / pathsep / name constants, getcwd returns str,
# getpid()>0, getenv defaults. os.path — hasattr join/exists/isdir/
# isfile/abspath/basename/dirname/split/splitext/expanduser/realpath
# + join/basename/dirname/split/splitext/exists/isdir/isfile value
# contracts. platform — hasattr system/machine/release/python_
# version/node + system/machine/python_version/node return str,
# python_version has dotted form. gc — hasattr enable/disable/
# collect/isenabled/get_count/get_threshold/set_threshold/get_
# objects/DEBUG_STATS + types for isenabled/collect/get_threshold/
# get_count, len-3 tuples, disable/enable roundtrip) are covered in
# the matching pass fixture `test_sys_os_platform_gc_value_ops`.
import sys
import os
import platform
import gc
from typing import Any


_ledger: list[int] = []

# 1) sys.platform == 'darwin' on macOS
#    (mamba: returns 'macos')
assert sys.platform == "darwin"; _ledger.append(1)

# 2) sys.version_info is integer-indexable
#    (mamba: raises KeyError '0')
def _vi_index_zero() -> Any:
    try:
        return sys.version_info[0]
    except KeyError:
        return None
assert _vi_index_zero() == 3; _ledger.append(1)

# 3) type(sys.float_info).__name__ == 'float_info'
#    (mamba: returns 'dict')
assert type(sys.float_info).__name__ == "float_info"; _ledger.append(1)

# 4) type(os.environ).__name__ == '_Environ'
#    (mamba: returns 'dict')
assert type(os.environ).__name__ == "_Environ"; _ledger.append(1)

# 5) 'HOME' in os.environ (HOME is set in the run environment)
#    (mamba: returns False — environ dict is empty on probe)
assert ("HOME" in os.environ) == True; _ledger.append(1)

# 6) hasattr(os.path, 'isabs') — os.path exposes isabs()
#    (mamba: returns False)
assert hasattr(os.path, "isabs") == True; _ledger.append(1)

# 7) platform.system() == 'Darwin' on macOS
#    (mamba: returns 'macos')
assert platform.system() == "Darwin"; _ledger.append(1)

# 8) platform.machine() == 'arm64' on Apple Silicon
#    (mamba: returns 'aarch64')
assert platform.machine() == "arm64"; _ledger.append(1)

# 9) hasattr(platform, 'python_implementation')
#    (mamba: returns False)
assert hasattr(platform, "python_implementation") == True; _ledger.append(1)

# 10) hasattr(gc, 'garbage') — gc exposes uncollectable list
#     (mamba: returns False)
assert hasattr(gc, "garbage") == True; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_sys_os_platform_gc_silent {sum(_ledger)} asserts")
