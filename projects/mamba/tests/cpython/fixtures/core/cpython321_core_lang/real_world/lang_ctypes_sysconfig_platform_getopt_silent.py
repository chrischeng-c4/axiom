# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "cpython321_core_lang"
# dimension = "real_world"
# case = "lang_ctypes_sysconfig_platform_getopt_silent"
# subject = "cpython321.lang_ctypes_sysconfig_platform_getopt_silent"
# kind = "semantic"
# xfail = "CPython 3.12 seed spec; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/spec/lang_ctypes_sysconfig_platform_getopt_silent.py"
# status = "filled"
# ///
"""cpython321.lang_ctypes_sysconfig_platform_getopt_silent: execute CPython 3.12 seed lang_ctypes_sysconfig_platform_getopt_silent"""
# mamba-xfail: CPython 3.12 seed spec; mamba promotion pending
# Operational AssertionPass divergence-spec fixture for the silent
# value-contract divergences of `ctypes.c_int(5).value` (the
# documented "c_int instance .value returns the wrapped int" — mamba
# returns None — value attribute is None), `ctypes.c_double(1.5).
# value` (the documented "c_double instance .value returns the
# wrapped float" — mamba returns None), `ctypes.sizeof(ctypes.c_int)
# == 4` (the documented "ctypes.sizeof of c_int returns 4 bytes" —
# mamba returns {} — sizeof of a dict-backed type), `hasattr
# (sysconfig, 'parse_config_h')` (the documented "sysconfig exposes
# the parse_config_h helper" — mamba returns False), `hasattr
# (platform, 'version')` (the documented "platform exposes the
# version() identity helper" — mamba returns False), `hasattr
# (platform, 'architecture')` (the documented "platform exposes the
# architecture() helper" — mamba returns False), `hasattr(platform,
# 'python_version_tuple')` (the documented "platform exposes the
# python_version_tuple() helper" — mamba returns False), `hasattr
# (platform, 'python_implementation')` (the documented "platform
# exposes the python_implementation() helper" — mamba returns
# False), `hasattr(platform, 'uname')` (the documented "platform
# exposes the uname() helper" — mamba returns False), and `hasattr
# (getopt, 'error')` (the documented "getopt exposes the error
# alias for GetoptError" — mamba returns False).
# Ten-pack pinned to atomic 285.
#
# Behavioral edges that CONFORM on mamba (ctypes — hasattr c_int/
# c_long/c_char/c_double/c_float/c_bool/c_void_p/c_char_p/c_wchar_p/
# c_uint + hasattr Structure/Union/Array/POINTER/pointer/byref/cast/
# sizeof/addressof/CDLL/ArgumentError. sysconfig — hasattr
# get_python_version/get_platform/get_paths/get_path/get_config_vars/
# get_config_var/get_path_names/get_scheme_names + str returns.
# platform — hasattr system/machine/release/platform/node/processor/
# python_version + str returns. getopt — hasattr getopt/gnu_getopt/
# GetoptError + getopt parses ['-a', 'x']) are covered in the
# matching pass fixture `test_ctypes_sysconfig_platform_getopt_
# value_ops`.
import ctypes
import sysconfig
import platform
import getopt


_ledger: list[int] = []

# 1) ctypes.c_int(5).value == 5 — wrapped int round-trip
#    (mamba: returns None — .value attribute is None)
assert ctypes.c_int(5).value == 5; _ledger.append(1)

# 2) ctypes.c_double(1.5).value == 1.5 — wrapped float round-trip
#    (mamba: returns None)
assert ctypes.c_double(1.5).value == 1.5; _ledger.append(1)

# 3) ctypes.sizeof(ctypes.c_int) == 4 — c_int is 4 bytes
#    (mamba: returns {} — sizeof returns dict, not int)
assert ctypes.sizeof(ctypes.c_int) == 4; _ledger.append(1)

# 4) hasattr(sysconfig, 'parse_config_h') — config-header parser
#    (mamba: returns False)
assert hasattr(sysconfig, "parse_config_h") == True; _ledger.append(1)

# 5) hasattr(platform, 'version') — OS version string helper
#    (mamba: returns False)
assert hasattr(platform, "version") == True; _ledger.append(1)

# 6) hasattr(platform, 'architecture') — architecture() tuple helper
#    (mamba: returns False)
assert hasattr(platform, "architecture") == True; _ledger.append(1)

# 7) hasattr(platform, 'python_version_tuple') — version tuple helper
#    (mamba: returns False)
assert hasattr(platform, "python_version_tuple") == True; _ledger.append(1)

# 8) hasattr(platform, 'python_implementation') — impl name helper
#    (mamba: returns False)
assert hasattr(platform, "python_implementation") == True; _ledger.append(1)

# 9) hasattr(platform, 'uname') — uname() namedtuple helper
#    (mamba: returns False)
assert hasattr(platform, "uname") == True; _ledger.append(1)

# 10) hasattr(getopt, 'error') — GetoptError alias
#     (mamba: returns False)
assert hasattr(getopt, "error") == True; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_ctypes_sysconfig_platform_getopt_silent {sum(_ledger)} asserts")
