# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "cpython321_builtins"
# dimension = "real_world"
# case = "test_string_struct_weakref_sys_value_ops"
# subject = "cpython321.test_string_struct_weakref_sys_value_ops"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/test_string_struct_weakref_sys_value_ops.py"
# status = "filled"
# ///
"""cpython321.test_string_struct_weakref_sys_value_ops: execute CPython 3.12 seed test_string_struct_weakref_sys_value_ops"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Atomic 318 pass conformance — string module (hasattr ascii_letters/
# ascii_lowercase/ascii_uppercase/digits/hexdigits/octdigits/
# punctuation/whitespace/capwords + string-constant values) + struct
# module (hasattr pack/unpack/calcsize/Struct/pack_into/unpack_from/
# iter_unpack/error + calcsize('i')==4 + calcsize('q')==8 + calcsize(
# 'd')==8) + weakref module (hasattr ref/proxy/WeakValueDictionary/
# WeakKeyDictionary/WeakSet/WeakMethod/finalize/getweakrefs/getweak
# refcount/CallableProxyType/ProxyType/ReferenceType/ProxyTypes) + gc
# module (hasattr collect/disable/enable/isenabled/get_objects/get_
# count/get_threshold/set_threshold/is_tracked/freeze/unfreeze/get_
# freeze_count/get_stats + DEBUG_STATS/DEBUG_COLLECTABLE/DEBUG_UNCOLL
# ECTABLE/DEBUG_SAVEALL/DEBUG_LEAK) + contextlib module (hasattr
# contextmanager/suppress/nullcontext) + sys module (hasattr argv/path
# /modules/stdin/stdout/stderr/platform/version/version_info/maxsize/
# byteorder/exit/exc_info/settrace/gettrace/getrecursionlimit/setrec
# ursionlimit/getsizeof/intern/getrefcount/hexversion/api_version/
# float_info/int_info/executable/prefix/exec_prefix/base_prefix/base_
# exec_prefix + byteorder=='little').
# All asserts match between CPython 3.12 and mamba.
import string
import struct
import weakref
import gc
import contextlib
import sys


_ledger: list[int] = []

# 1) string — hasattr (conformant subset) + string values
assert hasattr(string, "ascii_letters") == True; _ledger.append(1)
assert hasattr(string, "ascii_lowercase") == True; _ledger.append(1)
assert hasattr(string, "ascii_uppercase") == True; _ledger.append(1)
assert hasattr(string, "digits") == True; _ledger.append(1)
assert hasattr(string, "hexdigits") == True; _ledger.append(1)
assert hasattr(string, "octdigits") == True; _ledger.append(1)
assert hasattr(string, "punctuation") == True; _ledger.append(1)
assert hasattr(string, "whitespace") == True; _ledger.append(1)
assert hasattr(string, "capwords") == True; _ledger.append(1)
assert string.ascii_lowercase == "abcdefghijklmnopqrstuvwxyz"; _ledger.append(1)
assert string.ascii_uppercase == "ABCDEFGHIJKLMNOPQRSTUVWXYZ"; _ledger.append(1)
assert string.digits == "0123456789"; _ledger.append(1)
assert string.hexdigits == "0123456789abcdefABCDEF"; _ledger.append(1)
assert string.octdigits == "01234567"; _ledger.append(1)

# 2) struct — hasattr + calcsize value contracts
assert hasattr(struct, "pack") == True; _ledger.append(1)
assert hasattr(struct, "unpack") == True; _ledger.append(1)
assert hasattr(struct, "calcsize") == True; _ledger.append(1)
assert hasattr(struct, "Struct") == True; _ledger.append(1)
assert hasattr(struct, "pack_into") == True; _ledger.append(1)
assert hasattr(struct, "unpack_from") == True; _ledger.append(1)
assert hasattr(struct, "iter_unpack") == True; _ledger.append(1)
assert hasattr(struct, "error") == True; _ledger.append(1)
assert struct.calcsize("i") == 4; _ledger.append(1)
assert struct.calcsize("q") == 8; _ledger.append(1)
assert struct.calcsize("d") == 8; _ledger.append(1)

# 3) weakref — hasattr core surface
assert hasattr(weakref, "ref") == True; _ledger.append(1)
assert hasattr(weakref, "proxy") == True; _ledger.append(1)
assert hasattr(weakref, "WeakValueDictionary") == True; _ledger.append(1)
assert hasattr(weakref, "WeakKeyDictionary") == True; _ledger.append(1)
assert hasattr(weakref, "WeakSet") == True; _ledger.append(1)
assert hasattr(weakref, "WeakMethod") == True; _ledger.append(1)
assert hasattr(weakref, "finalize") == True; _ledger.append(1)
assert hasattr(weakref, "getweakrefs") == True; _ledger.append(1)
assert hasattr(weakref, "getweakrefcount") == True; _ledger.append(1)
assert hasattr(weakref, "CallableProxyType") == True; _ledger.append(1)
assert hasattr(weakref, "ProxyType") == True; _ledger.append(1)
assert hasattr(weakref, "ReferenceType") == True; _ledger.append(1)
assert hasattr(weakref, "ProxyTypes") == True; _ledger.append(1)

# 4) gc — hasattr (conformant subset)
assert hasattr(gc, "collect") == True; _ledger.append(1)
assert hasattr(gc, "disable") == True; _ledger.append(1)
assert hasattr(gc, "enable") == True; _ledger.append(1)
assert hasattr(gc, "isenabled") == True; _ledger.append(1)
assert hasattr(gc, "get_objects") == True; _ledger.append(1)
assert hasattr(gc, "get_count") == True; _ledger.append(1)
assert hasattr(gc, "get_threshold") == True; _ledger.append(1)
assert hasattr(gc, "set_threshold") == True; _ledger.append(1)
assert hasattr(gc, "is_tracked") == True; _ledger.append(1)
assert hasattr(gc, "freeze") == True; _ledger.append(1)
assert hasattr(gc, "unfreeze") == True; _ledger.append(1)
assert hasattr(gc, "get_freeze_count") == True; _ledger.append(1)
assert hasattr(gc, "get_stats") == True; _ledger.append(1)
assert hasattr(gc, "DEBUG_STATS") == True; _ledger.append(1)
assert hasattr(gc, "DEBUG_COLLECTABLE") == True; _ledger.append(1)
assert hasattr(gc, "DEBUG_UNCOLLECTABLE") == True; _ledger.append(1)
assert hasattr(gc, "DEBUG_SAVEALL") == True; _ledger.append(1)
assert hasattr(gc, "DEBUG_LEAK") == True; _ledger.append(1)

# 5) contextlib — hasattr (conformant subset)
assert hasattr(contextlib, "contextmanager") == True; _ledger.append(1)
assert hasattr(contextlib, "suppress") == True; _ledger.append(1)
assert hasattr(contextlib, "nullcontext") == True; _ledger.append(1)

# 6) sys — hasattr (conformant subset)
assert hasattr(sys, "argv") == True; _ledger.append(1)
assert hasattr(sys, "path") == True; _ledger.append(1)
assert hasattr(sys, "modules") == True; _ledger.append(1)
assert hasattr(sys, "stdin") == True; _ledger.append(1)
assert hasattr(sys, "stdout") == True; _ledger.append(1)
assert hasattr(sys, "stderr") == True; _ledger.append(1)
assert hasattr(sys, "platform") == True; _ledger.append(1)
assert hasattr(sys, "version") == True; _ledger.append(1)
assert hasattr(sys, "version_info") == True; _ledger.append(1)
assert hasattr(sys, "maxsize") == True; _ledger.append(1)
assert hasattr(sys, "byteorder") == True; _ledger.append(1)
assert hasattr(sys, "exit") == True; _ledger.append(1)
assert hasattr(sys, "exc_info") == True; _ledger.append(1)
assert hasattr(sys, "settrace") == True; _ledger.append(1)
assert hasattr(sys, "gettrace") == True; _ledger.append(1)
assert hasattr(sys, "getrecursionlimit") == True; _ledger.append(1)
assert hasattr(sys, "setrecursionlimit") == True; _ledger.append(1)
assert hasattr(sys, "getsizeof") == True; _ledger.append(1)
assert hasattr(sys, "intern") == True; _ledger.append(1)
assert hasattr(sys, "getrefcount") == True; _ledger.append(1)
assert hasattr(sys, "hexversion") == True; _ledger.append(1)
assert hasattr(sys, "api_version") == True; _ledger.append(1)
assert hasattr(sys, "float_info") == True; _ledger.append(1)
assert hasattr(sys, "int_info") == True; _ledger.append(1)
assert hasattr(sys, "executable") == True; _ledger.append(1)
assert hasattr(sys, "prefix") == True; _ledger.append(1)
assert hasattr(sys, "exec_prefix") == True; _ledger.append(1)
assert hasattr(sys, "base_prefix") == True; _ledger.append(1)
assert hasattr(sys, "base_exec_prefix") == True; _ledger.append(1)
assert sys.byteorder == "little"; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_string_struct_weakref_sys_value_ops {sum(_ledger)} asserts")
