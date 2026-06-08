# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "cpython321_core_lang"
# dimension = "real_world"
# case = "lang_string_gc_contextlib_sys_silent"
# subject = "cpython321.lang_string_gc_contextlib_sys_silent"
# kind = "semantic"
# xfail = "CPython 3.12 seed spec; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/spec/lang_string_gc_contextlib_sys_silent.py"
# status = "filled"
# ///
"""cpython321.lang_string_gc_contextlib_sys_silent: execute CPython 3.12 seed lang_string_gc_contextlib_sys_silent"""
# mamba-xfail: CPython 3.12 seed spec; mamba promotion pending
# Operational AssertionPass divergence-spec fixture for the silent
# value-contract divergences of `hasattr(string, 'printable')` (the
# documented "string exposes the printable character set" — mamba
# returns False), `hasattr(sys, 'maxunicode')` (the documented "sys
# exposes the maxunicode codepoint limit" — mamba returns False),
# `sys.maxsize == 9223372036854775807` (the documented "sys.maxsize is
# 2**63-1 on 64-bit platforms" — mamba returns 140737488355327 —
# 48-bit integer cap), `sys.platform == 'darwin'` (the documented
# "sys.platform is 'darwin' on macOS" — mamba returns 'macos' —
# platform-string divergence), `hasattr(gc, 'set_debug')` (the
# documented "gc exposes the set_debug helper" — mamba returns False),
# `hasattr(gc, 'garbage')` (the documented "gc exposes the garbage
# list" — mamba returns False), `hasattr(gc, 'callbacks')` (the
# documented "gc exposes the callbacks list" — mamba returns False),
# `hasattr(contextlib, 'closing')` (the documented "contextlib exposes
# the closing context manager" — mamba returns False), `hasattr(
# contextlib, 'ExitStack')` (the documented "contextlib exposes the
# ExitStack class" — mamba returns False), and `hasattr(contextlib,
# 'redirect_stdout')` (the documented "contextlib exposes the
# redirect_stdout context manager" — mamba returns False).
# Ten-pack pinned to atomic 318.
#
# Behavioral edges that CONFORM on mamba (string — hasattr ascii_
# letters/ascii_lowercase/ascii_uppercase/digits/hexdigits/octdigits/
# punctuation/whitespace/capwords + string-constant values. struct —
# hasattr pack/unpack/calcsize/Struct/pack_into/unpack_from/iter_
# unpack/error + calcsize('i')==4 + calcsize('q')==8 + calcsize('d')
# ==8. weakref — hasattr ref/proxy/WeakValueDictionary/WeakKey
# Dictionary/WeakSet/WeakMethod/finalize/getweakrefs/getweakrefcount/
# CallableProxyType/ProxyType/ReferenceType/ProxyTypes. gc — hasattr
# collect/disable/enable/isenabled/get_objects/get_count/get_threshold
# /set_threshold/is_tracked/freeze/unfreeze/get_freeze_count/get_stats
# + DEBUG_STATS/DEBUG_COLLECTABLE/DEBUG_UNCOLLECTABLE/DEBUG_SAVEALL/
# DEBUG_LEAK. contextlib — hasattr contextmanager/suppress/null
# context. sys — hasattr argv/path/modules/stdin/stdout/stderr/
# platform/version/version_info/maxsize/byteorder/exit/exc_info/
# settrace/gettrace/getrecursionlimit/setrecursionlimit/getsizeof/
# intern/getrefcount/hexversion/api_version/float_info/int_info/
# executable/prefix/exec_prefix/base_prefix/base_exec_prefix +
# byteorder=='little') are covered in the matching pass fixture
# `test_string_struct_weakref_sys_value_ops`.
import string
import sys
import gc
import contextlib


_ledger: list[int] = []

# 1) hasattr(string, 'printable') — printable character set
#    (mamba: returns False)
assert hasattr(string, "printable") == True; _ledger.append(1)

# 2) hasattr(sys, 'maxunicode') — maxunicode codepoint limit
#    (mamba: returns False)
assert hasattr(sys, "maxunicode") == True; _ledger.append(1)

# 3) sys.maxsize == 9223372036854775807 — 2**63-1 on 64-bit platforms
#    (mamba: returns 140737488355327 — 48-bit integer cap)
assert sys.maxsize == 9223372036854775807; _ledger.append(1)

# 4) sys.platform == 'darwin' — macOS platform string
#    (mamba: returns 'macos' — platform-string divergence)
assert sys.platform == "darwin"; _ledger.append(1)

# 5) hasattr(gc, 'set_debug') — set_debug helper
#    (mamba: returns False)
assert hasattr(gc, "set_debug") == True; _ledger.append(1)

# 6) hasattr(gc, 'garbage') — garbage list
#    (mamba: returns False)
assert hasattr(gc, "garbage") == True; _ledger.append(1)

# 7) hasattr(gc, 'callbacks') — callbacks list
#    (mamba: returns False)
assert hasattr(gc, "callbacks") == True; _ledger.append(1)

# 8) hasattr(contextlib, 'closing') — closing context manager
#    (mamba: returns False)
assert hasattr(contextlib, "closing") == True; _ledger.append(1)

# 9) hasattr(contextlib, 'ExitStack') — ExitStack class
#    (mamba: returns False)
assert hasattr(contextlib, "ExitStack") == True; _ledger.append(1)

# 10) hasattr(contextlib, 'redirect_stdout') — redirect_stdout context manager
#     (mamba: returns False)
assert hasattr(contextlib, "redirect_stdout") == True; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_string_gc_contextlib_sys_silent {sum(_ledger)} asserts")
