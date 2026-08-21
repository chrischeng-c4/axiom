# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cpython321_stdlib"
# dimension = "real_world"
# case = "test_json_copy_pickle_weakref_abc_gc_value_ops"
# subject = "cpython321.test_json_copy_pickle_weakref_abc_gc_value_ops"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/test_json_copy_pickle_weakref_abc_gc_value_ops.py"
# status = "filled"
# ///
"""cpython321.test_json_copy_pickle_weakref_abc_gc_value_ops: execute CPython 3.12 seed test_json_copy_pickle_weakref_abc_gc_value_ops"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Atomic 228 pass conformance — json/copy/pickle/weakref/types/abc/gc/
# atexit/signal/string/re hasattr + value ops that match between
# CPython 3.12 and mamba.
import json
import copy
import pickle
import weakref
import types
import abc
import gc
import atexit
import signal
import string
import re

_ledger: list[int] = []

# 1) json — round-trip value ops
assert json.dumps({"a": 1, "b": 2}) == '{"a": 1, "b": 2}'; _ledger.append(1)
assert json.dumps([1, 2, 3]) == "[1, 2, 3]"; _ledger.append(1)
assert json.loads('{"x": 1}') == {"x": 1}; _ledger.append(1)
assert json.loads("[1, 2, 3]") == [1, 2, 3]; _ledger.append(1)
assert json.loads("true") == True; _ledger.append(1)
assert json.loads("false") == False; _ledger.append(1)
assert json.loads("null") is None; _ledger.append(1)
assert json.loads("42") == 42; _ledger.append(1)

# 2) copy — value-equivalence ops
_src = [1, 2, [3, 4]]
_shallow = copy.copy(_src)
assert _shallow == [1, 2, [3, 4]]; _ledger.append(1)
_deep = copy.deepcopy(_src)
assert _deep == [1, 2, [3, 4]]; _ledger.append(1)
assert _deep[2] is not _src[2]; _ledger.append(1)
assert hasattr(copy, "copy") == True; _ledger.append(1)
assert hasattr(copy, "deepcopy") == True; _ledger.append(1)
assert hasattr(copy, "Error") == True; _ledger.append(1)

# 3) pickle — round-trip + module hasattr surface
_b = pickle.dumps([1, 2, 3])
assert isinstance(_b, bytes); _ledger.append(1)
assert pickle.loads(_b) == [1, 2, 3]; _ledger.append(1)
assert pickle.loads(pickle.dumps({"a": 1, "b": 2})) == {"a": 1, "b": 2}; _ledger.append(1)
assert pickle.loads(pickle.dumps((1, 2, 3))) == (1, 2, 3); _ledger.append(1)
assert hasattr(pickle, "dumps") == True; _ledger.append(1)
assert hasattr(pickle, "loads") == True; _ledger.append(1)
assert hasattr(pickle, "dump") == True; _ledger.append(1)
assert hasattr(pickle, "load") == True; _ledger.append(1)
assert hasattr(pickle, "Pickler") == True; _ledger.append(1)
assert hasattr(pickle, "Unpickler") == True; _ledger.append(1)
assert hasattr(pickle, "HIGHEST_PROTOCOL") == True; _ledger.append(1)
assert hasattr(pickle, "DEFAULT_PROTOCOL") == True; _ledger.append(1)
assert hasattr(pickle, "PickleError") == True; _ledger.append(1)
assert hasattr(pickle, "PicklingError") == True; _ledger.append(1)
assert hasattr(pickle, "UnpicklingError") == True; _ledger.append(1)

# 4) weakref — full hasattr surface
assert hasattr(weakref, "ref") == True; _ledger.append(1)
assert hasattr(weakref, "proxy") == True; _ledger.append(1)
assert hasattr(weakref, "WeakValueDictionary") == True; _ledger.append(1)
assert hasattr(weakref, "WeakKeyDictionary") == True; _ledger.append(1)
assert hasattr(weakref, "WeakSet") == True; _ledger.append(1)
assert hasattr(weakref, "finalize") == True; _ledger.append(1)
assert hasattr(weakref, "ReferenceType") == True; _ledger.append(1)
assert hasattr(weakref, "ProxyType") == True; _ledger.append(1)

# 5) types — top-level type-name hasattr surface
assert hasattr(types, "FunctionType") == True; _ledger.append(1)
assert hasattr(types, "LambdaType") == True; _ledger.append(1)
assert hasattr(types, "GeneratorType") == True; _ledger.append(1)
assert hasattr(types, "CoroutineType") == True; _ledger.append(1)
assert hasattr(types, "AsyncGeneratorType") == True; _ledger.append(1)
assert hasattr(types, "MethodType") == True; _ledger.append(1)
assert hasattr(types, "BuiltinFunctionType") == True; _ledger.append(1)
assert hasattr(types, "BuiltinMethodType") == True; _ledger.append(1)
assert hasattr(types, "ModuleType") == True; _ledger.append(1)
assert hasattr(types, "TracebackType") == True; _ledger.append(1)
assert hasattr(types, "FrameType") == True; _ledger.append(1)
assert hasattr(types, "CodeType") == True; _ledger.append(1)
assert hasattr(types, "MappingProxyType") == True; _ledger.append(1)
assert hasattr(types, "GenericAlias") == True; _ledger.append(1)
assert hasattr(types, "UnionType") == True; _ledger.append(1)
assert hasattr(types, "NoneType") == True; _ledger.append(1)

# 6) abc — full hasattr surface
assert hasattr(abc, "ABC") == True; _ledger.append(1)
assert hasattr(abc, "ABCMeta") == True; _ledger.append(1)
assert hasattr(abc, "abstractmethod") == True; _ledger.append(1)
assert hasattr(abc, "abstractclassmethod") == True; _ledger.append(1)
assert hasattr(abc, "abstractstaticmethod") == True; _ledger.append(1)
assert hasattr(abc, "abstractproperty") == True; _ledger.append(1)
assert hasattr(abc, "update_abstractmethods") == True; _ledger.append(1)
assert hasattr(abc, "get_cache_token") == True; _ledger.append(1)

# 7) gc — common surface hasattr
assert hasattr(gc, "collect") == True; _ledger.append(1)
assert hasattr(gc, "enable") == True; _ledger.append(1)
assert hasattr(gc, "disable") == True; _ledger.append(1)
assert hasattr(gc, "isenabled") == True; _ledger.append(1)
assert hasattr(gc, "get_count") == True; _ledger.append(1)
assert hasattr(gc, "get_threshold") == True; _ledger.append(1)
assert hasattr(gc, "set_threshold") == True; _ledger.append(1)
assert hasattr(gc, "get_objects") == True; _ledger.append(1)
assert hasattr(gc, "DEBUG_STATS") == True; _ledger.append(1)

# 8) atexit — full hasattr surface
assert hasattr(atexit, "register") == True; _ledger.append(1)
assert hasattr(atexit, "unregister") == True; _ledger.append(1)
assert hasattr(atexit, "_run_exitfuncs") == True; _ledger.append(1)
assert hasattr(atexit, "_clear") == True; _ledger.append(1)
assert hasattr(atexit, "_ncallbacks") == True; _ledger.append(1)

# 9) signal — common surface hasattr
assert hasattr(signal, "signal") == True; _ledger.append(1)
assert hasattr(signal, "getsignal") == True; _ledger.append(1)
assert hasattr(signal, "SIGINT") == True; _ledger.append(1)
assert hasattr(signal, "SIGTERM") == True; _ledger.append(1)
assert hasattr(signal, "SIGKILL") == True; _ledger.append(1)
assert hasattr(signal, "SIG_DFL") == True; _ledger.append(1)
assert hasattr(signal, "SIG_IGN") == True; _ledger.append(1)
assert hasattr(signal, "Signals") == True; _ledger.append(1)
assert hasattr(signal, "Handlers") == True; _ledger.append(1)

# 10) string — letter/digit/punctuation constants and value contracts
assert string.ascii_lowercase == "abcdefghijklmnopqrstuvwxyz"; _ledger.append(1)
assert string.ascii_uppercase == "ABCDEFGHIJKLMNOPQRSTUVWXYZ"; _ledger.append(1)
assert string.ascii_letters == "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ"; _ledger.append(1)
assert string.digits == "0123456789"; _ledger.append(1)
assert string.hexdigits == "0123456789abcdefABCDEF"; _ledger.append(1)
assert string.octdigits == "01234567"; _ledger.append(1)
assert hasattr(string, "punctuation") == True; _ledger.append(1)
assert hasattr(string, "whitespace") == True; _ledger.append(1)
assert hasattr(string, "Template") == True; _ledger.append(1)
assert hasattr(string, "Formatter") == True; _ledger.append(1)
assert hasattr(string, "capwords") == True; _ledger.append(1)

# 11) re — common surface hasattr
assert hasattr(re, "match") == True; _ledger.append(1)
assert hasattr(re, "search") == True; _ledger.append(1)
assert hasattr(re, "findall") == True; _ledger.append(1)
assert hasattr(re, "finditer") == True; _ledger.append(1)
assert hasattr(re, "sub") == True; _ledger.append(1)
assert hasattr(re, "compile") == True; _ledger.append(1)
assert hasattr(re, "Pattern") == True; _ledger.append(1)
assert hasattr(re, "Match") == True; _ledger.append(1)
assert hasattr(re, "IGNORECASE") == True; _ledger.append(1)
assert hasattr(re, "MULTILINE") == True; _ledger.append(1)
assert hasattr(re, "DOTALL") == True; _ledger.append(1)
assert hasattr(re, "VERBOSE") == True; _ledger.append(1)
assert hasattr(re, "ASCII") == True; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_json_copy_pickle_weakref_abc_gc_value_ops {sum(_ledger)} asserts")
