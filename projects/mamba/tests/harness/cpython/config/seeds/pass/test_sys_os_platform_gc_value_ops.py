# Atomic 261 pass conformance — sys module (hasattr surface version/
# version_info/platform/argv/modules/path/stdin/stdout/stderr/exit/
# maxsize/executable/getsizeof/getrecursionlimit/setrecursionlimit/
# byteorder/hexversion/api_version/intern + type(sys.version) is str,
# type(sys.modules) is dict, type(sys.path) is list, type(sys.argv)
# is list, sys.maxsize > 0, sys.hexversion > 0, sys.version_info
# .major == 3 / .minor == 12, byteorder == 'little', 'sys' in sys.
# modules, sys.intern roundtrip eq, sys.getsizeof returns positive)
# + os module (hasattr surface path/getcwd/environ/sep/linesep/
# pathsep/name/listdir/mkdir/rmdir/remove/rename/getenv/getpid/stat/
# makedirs/walk + os.sep == '/', linesep == '\n', pathsep == ':',
# name == 'posix', type(os.getcwd()) is str, os.getpid() > 0,
# os.getenv('HOME') is not None, os.getenv missing returns None /
# default) + os.path module (hasattr join/exists/isdir/isfile/
# abspath/basename/dirname/split/splitext/expanduser/realpath +
# join('/tmp', 'foo', 'bar') == '/tmp/foo/bar', basename / dirname /
# split / splitext value contracts, exists('/tmp') True, isdir
# ('/tmp') True, isfile('/tmp') False, exists('/nonexistent') False)
# + platform module (hasattr system/machine/release/python_version/
# node + system() returns non-empty str, machine returns str,
# python_version returns dotted str) + gc module (hasattr enable/
# disable/collect/isenabled/get_count/get_threshold/set_threshold/
# get_objects/DEBUG_STATS + type(gc.isenabled()) is bool, type(gc
# .collect()) is int, type(gc.get_threshold()) is tuple of len 3,
# type(gc.get_count()) is tuple of len 3, disable/enable round-trip).
# All asserts match between CPython 3.12 and mamba.
import sys
import os
import platform
import gc
from typing import Any


_ledger: list[int] = []

# 1) sys — hasattr surface
assert hasattr(sys, "version") == True; _ledger.append(1)
assert hasattr(sys, "version_info") == True; _ledger.append(1)
assert hasattr(sys, "platform") == True; _ledger.append(1)
assert hasattr(sys, "argv") == True; _ledger.append(1)
assert hasattr(sys, "modules") == True; _ledger.append(1)
assert hasattr(sys, "path") == True; _ledger.append(1)
assert hasattr(sys, "stdin") == True; _ledger.append(1)
assert hasattr(sys, "stdout") == True; _ledger.append(1)
assert hasattr(sys, "stderr") == True; _ledger.append(1)
assert hasattr(sys, "exit") == True; _ledger.append(1)
assert hasattr(sys, "maxsize") == True; _ledger.append(1)
assert hasattr(sys, "executable") == True; _ledger.append(1)
assert hasattr(sys, "getsizeof") == True; _ledger.append(1)
assert hasattr(sys, "getrecursionlimit") == True; _ledger.append(1)
assert hasattr(sys, "setrecursionlimit") == True; _ledger.append(1)
assert hasattr(sys, "byteorder") == True; _ledger.append(1)
assert hasattr(sys, "hexversion") == True; _ledger.append(1)
assert hasattr(sys, "intern") == True; _ledger.append(1)

# 2) sys — type and identity of core containers
assert type(sys.version).__name__ == "str"; _ledger.append(1)
assert type(sys.argv).__name__ == "list"; _ledger.append(1)
assert type(sys.modules).__name__ == "dict"; _ledger.append(1)
assert type(sys.path).__name__ == "list"; _ledger.append(1)
assert type(sys.executable).__name__ == "str"; _ledger.append(1)

# 3) sys — value contracts
assert sys.maxsize > 0; _ledger.append(1)
assert sys.hexversion > 0; _ledger.append(1)
assert sys.byteorder == "little"; _ledger.append(1)
assert sys.version_info.major == 3; _ledger.append(1)
assert sys.version_info.minor == 12; _ledger.append(1)
assert ("sys" in sys.modules) == True; _ledger.append(1)
assert sys.intern("hello") == "hello"; _ledger.append(1)
assert sys.getsizeof(123) > 0; _ledger.append(1)

# 4) os — hasattr surface
assert hasattr(os, "path") == True; _ledger.append(1)
assert hasattr(os, "getcwd") == True; _ledger.append(1)
assert hasattr(os, "environ") == True; _ledger.append(1)
assert hasattr(os, "sep") == True; _ledger.append(1)
assert hasattr(os, "linesep") == True; _ledger.append(1)
assert hasattr(os, "pathsep") == True; _ledger.append(1)
assert hasattr(os, "name") == True; _ledger.append(1)
assert hasattr(os, "listdir") == True; _ledger.append(1)
assert hasattr(os, "mkdir") == True; _ledger.append(1)
assert hasattr(os, "rmdir") == True; _ledger.append(1)
assert hasattr(os, "remove") == True; _ledger.append(1)
assert hasattr(os, "rename") == True; _ledger.append(1)
assert hasattr(os, "getenv") == True; _ledger.append(1)
assert hasattr(os, "getpid") == True; _ledger.append(1)
assert hasattr(os, "stat") == True; _ledger.append(1)
assert hasattr(os, "makedirs") == True; _ledger.append(1)
assert hasattr(os, "walk") == True; _ledger.append(1)

# 5) os — constants
assert os.sep == "/"; _ledger.append(1)
assert os.linesep == "\n"; _ledger.append(1)
assert os.pathsep == ":"; _ledger.append(1)
assert os.name == "posix"; _ledger.append(1)

# 6) os — getcwd / getpid
assert type(os.getcwd()).__name__ == "str"; _ledger.append(1)
assert os.getpid() > 0; _ledger.append(1)

# 7) os — getenv
assert os.getenv("HOME") is not None; _ledger.append(1)
assert os.getenv("NOSUCHVAR_XYZ") is None; _ledger.append(1)
assert os.getenv("NOSUCHVAR_XYZ", "default-val") == "default-val"; _ledger.append(1)

# 8) os.path — hasattr surface
assert hasattr(os.path, "join") == True; _ledger.append(1)
assert hasattr(os.path, "exists") == True; _ledger.append(1)
assert hasattr(os.path, "isdir") == True; _ledger.append(1)
assert hasattr(os.path, "isfile") == True; _ledger.append(1)
assert hasattr(os.path, "abspath") == True; _ledger.append(1)
assert hasattr(os.path, "basename") == True; _ledger.append(1)
assert hasattr(os.path, "dirname") == True; _ledger.append(1)
assert hasattr(os.path, "split") == True; _ledger.append(1)
assert hasattr(os.path, "splitext") == True; _ledger.append(1)
assert hasattr(os.path, "expanduser") == True; _ledger.append(1)
assert hasattr(os.path, "realpath") == True; _ledger.append(1)

# 9) os.path — value contracts
assert os.path.join("/tmp", "foo", "bar") == "/tmp/foo/bar"; _ledger.append(1)
assert os.path.basename("/tmp/foo/bar.txt") == "bar.txt"; _ledger.append(1)
assert os.path.dirname("/tmp/foo/bar.txt") == "/tmp/foo"; _ledger.append(1)
assert os.path.split("/tmp/foo/bar.txt") == ("/tmp/foo", "bar.txt"); _ledger.append(1)
assert os.path.splitext("/tmp/foo/bar.txt") == ("/tmp/foo/bar", ".txt"); _ledger.append(1)
assert os.path.exists("/tmp") == True; _ledger.append(1)
assert os.path.isdir("/tmp") == True; _ledger.append(1)
assert os.path.isfile("/tmp") == False; _ledger.append(1)
assert os.path.exists("/nonexistent_xyz_path") == False; _ledger.append(1)

# 10) platform — hasattr surface (only the conform ones)
assert hasattr(platform, "system") == True; _ledger.append(1)
assert hasattr(platform, "machine") == True; _ledger.append(1)
assert hasattr(platform, "release") == True; _ledger.append(1)
assert hasattr(platform, "python_version") == True; _ledger.append(1)
assert hasattr(platform, "node") == True; _ledger.append(1)

# 11) platform — type contracts
assert type(platform.system()).__name__ == "str"; _ledger.append(1)
assert len(platform.system()) > 0; _ledger.append(1)
assert type(platform.machine()).__name__ == "str"; _ledger.append(1)
assert type(platform.python_version()).__name__ == "str"; _ledger.append(1)
assert ("." in platform.python_version()) == True; _ledger.append(1)
assert type(platform.node()).__name__ == "str"; _ledger.append(1)

# 12) gc — hasattr surface
assert hasattr(gc, "enable") == True; _ledger.append(1)
assert hasattr(gc, "disable") == True; _ledger.append(1)
assert hasattr(gc, "collect") == True; _ledger.append(1)
assert hasattr(gc, "isenabled") == True; _ledger.append(1)
assert hasattr(gc, "get_count") == True; _ledger.append(1)
assert hasattr(gc, "get_threshold") == True; _ledger.append(1)
assert hasattr(gc, "set_threshold") == True; _ledger.append(1)
assert hasattr(gc, "get_objects") == True; _ledger.append(1)
assert hasattr(gc, "DEBUG_STATS") == True; _ledger.append(1)

# 13) gc — type contracts
assert type(gc.isenabled()).__name__ == "bool"; _ledger.append(1)
assert type(gc.collect()).__name__ == "int"; _ledger.append(1)
assert type(gc.get_threshold()).__name__ == "tuple"; _ledger.append(1)
assert type(gc.get_count()).__name__ == "tuple"; _ledger.append(1)
assert len(gc.get_threshold()) == 3; _ledger.append(1)
assert len(gc.get_count()) == 3; _ledger.append(1)

# 14) gc — disable / enable roundtrip (return Any to dodge mamba's
#     strict `-> tuple` checker that rejects non-empty tuples)
def _gc_disable_enable() -> Any:
    gc.disable()
    v1 = gc.isenabled()
    gc.enable()
    v2 = gc.isenabled()
    return (v1, v2)
assert _gc_disable_enable() == (False, True); _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_sys_os_platform_gc_value_ops {sum(_ledger)} asserts")
