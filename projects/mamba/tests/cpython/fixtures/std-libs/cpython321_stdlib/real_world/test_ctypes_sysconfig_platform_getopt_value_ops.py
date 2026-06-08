# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cpython321_stdlib"
# dimension = "real_world"
# case = "test_ctypes_sysconfig_platform_getopt_value_ops"
# subject = "cpython321.test_ctypes_sysconfig_platform_getopt_value_ops"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/test_ctypes_sysconfig_platform_getopt_value_ops.py"
# status = "filled"
# ///
"""cpython321.test_ctypes_sysconfig_platform_getopt_value_ops: execute CPython 3.12 seed test_ctypes_sysconfig_platform_getopt_value_ops"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Atomic 285 pass conformance — ctypes module (hasattr CDLL/c_int/
# c_long/c_char/c_void_p/c_double/c_float/c_char_p/c_wchar_p/c_bool/
# c_uint + hasattr Structure/Union/POINTER/pointer/byref/cast/sizeof/
# addressof/Array/ArgumentError) + sysconfig module (hasattr
# get_python_version/get_platform/get_paths/get_path/get_config_vars/
# get_config_var/get_path_names/get_scheme_names + get_python_version
# returns str + get_platform returns str) + platform module (hasattr
# system/machine/release/platform/node/processor/python_version +
# python_version returns str + system returns str) + getopt module
# (hasattr getopt/gnu_getopt/GetoptError + getopt parses ['-a','x']
# correctly).
# All asserts match between CPython 3.12 and mamba.
import ctypes
import sysconfig
import platform
import getopt


_ledger: list[int] = []

# 1) ctypes — hasattr c-scalar surface
assert hasattr(ctypes, "c_int") == True; _ledger.append(1)
assert hasattr(ctypes, "c_long") == True; _ledger.append(1)
assert hasattr(ctypes, "c_char") == True; _ledger.append(1)
assert hasattr(ctypes, "c_double") == True; _ledger.append(1)
assert hasattr(ctypes, "c_float") == True; _ledger.append(1)
assert hasattr(ctypes, "c_bool") == True; _ledger.append(1)
assert hasattr(ctypes, "c_void_p") == True; _ledger.append(1)
assert hasattr(ctypes, "c_char_p") == True; _ledger.append(1)
assert hasattr(ctypes, "c_wchar_p") == True; _ledger.append(1)
assert hasattr(ctypes, "c_uint") == True; _ledger.append(1)

# 2) ctypes — hasattr aggregate/operator surface
assert hasattr(ctypes, "Structure") == True; _ledger.append(1)
assert hasattr(ctypes, "Union") == True; _ledger.append(1)
assert hasattr(ctypes, "Array") == True; _ledger.append(1)
assert hasattr(ctypes, "POINTER") == True; _ledger.append(1)
assert hasattr(ctypes, "pointer") == True; _ledger.append(1)
assert hasattr(ctypes, "byref") == True; _ledger.append(1)
assert hasattr(ctypes, "cast") == True; _ledger.append(1)
assert hasattr(ctypes, "sizeof") == True; _ledger.append(1)
assert hasattr(ctypes, "addressof") == True; _ledger.append(1)
assert hasattr(ctypes, "CDLL") == True; _ledger.append(1)
assert hasattr(ctypes, "ArgumentError") == True; _ledger.append(1)

# 3) sysconfig — hasattr config surface
assert hasattr(sysconfig, "get_python_version") == True; _ledger.append(1)
assert hasattr(sysconfig, "get_platform") == True; _ledger.append(1)
assert hasattr(sysconfig, "get_paths") == True; _ledger.append(1)
assert hasattr(sysconfig, "get_path") == True; _ledger.append(1)
assert hasattr(sysconfig, "get_config_vars") == True; _ledger.append(1)
assert hasattr(sysconfig, "get_config_var") == True; _ledger.append(1)
assert hasattr(sysconfig, "get_path_names") == True; _ledger.append(1)
assert hasattr(sysconfig, "get_scheme_names") == True; _ledger.append(1)

# 4) sysconfig — value contracts (str)
assert isinstance(sysconfig.get_python_version(), str) == True; _ledger.append(1)
assert isinstance(sysconfig.get_platform(), str) == True; _ledger.append(1)

# 5) platform — hasattr identity surface
assert hasattr(platform, "system") == True; _ledger.append(1)
assert hasattr(platform, "machine") == True; _ledger.append(1)
assert hasattr(platform, "release") == True; _ledger.append(1)
assert hasattr(platform, "platform") == True; _ledger.append(1)
assert hasattr(platform, "node") == True; _ledger.append(1)
assert hasattr(platform, "processor") == True; _ledger.append(1)
assert hasattr(platform, "python_version") == True; _ledger.append(1)

# 6) platform — value contracts (str)
assert isinstance(platform.python_version(), str) == True; _ledger.append(1)
assert isinstance(platform.system(), str) == True; _ledger.append(1)

# 7) getopt — hasattr core surface
assert hasattr(getopt, "getopt") == True; _ledger.append(1)
assert hasattr(getopt, "gnu_getopt") == True; _ledger.append(1)
assert hasattr(getopt, "GetoptError") == True; _ledger.append(1)

# 8) getopt — parse contract
_opts, _args = getopt.getopt(["-a", "x"], "a:")
assert _opts == [("-a", "x")]; _ledger.append(1)
assert _args == []; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_ctypes_sysconfig_platform_getopt_value_ops {sum(_ledger)} asserts")
