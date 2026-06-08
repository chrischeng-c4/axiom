# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cpython321_stdlib"
# dimension = "real_world"
# case = "test_pickle_numbers_gc_atexit_value_ops"
# subject = "cpython321.test_pickle_numbers_gc_atexit_value_ops"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/test_pickle_numbers_gc_atexit_value_ops.py"
# status = "filled"
# ///
"""cpython321.test_pickle_numbers_gc_atexit_value_ops: execute CPython 3.12 seed test_pickle_numbers_gc_atexit_value_ops"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Atomic 303 pass conformance — pickle module (hasattr dumps/loads/
# dump/load/Pickler/Unpickler/PickleError/PicklingError/UnpicklingError/
# HIGHEST_PROTOCOL/DEFAULT_PROTOCOL + dumps returns bytes + dumps/loads
# round-trip int/str/list/dict) + numbers module (hasattr Number/
# Complex/Real/Rational/Integral) + ipaddress module (hasattr ip_
# address/ip_network/IPv4Address/IPv6Address/IPv4Network/IPv6Network/
# AddressValueError/NetmaskValueError) + locale module (hasattr
# getlocale/setlocale/LC_ALL/LC_CTYPE/LC_NUMERIC/LC_TIME) + gc module
# (hasattr collect/disable/enable/isenabled/get_count/get_threshold/
# set_threshold/DEBUG_LEAK) + atexit module (hasattr register/
# unregister).
# All asserts match between CPython 3.12 and mamba.
import pickle
import numbers
import ipaddress
import locale
import gc
import atexit


_ledger: list[int] = []

# 1) pickle — hasattr core surface
assert hasattr(pickle, "dumps") == True; _ledger.append(1)
assert hasattr(pickle, "loads") == True; _ledger.append(1)
assert hasattr(pickle, "dump") == True; _ledger.append(1)
assert hasattr(pickle, "load") == True; _ledger.append(1)
assert hasattr(pickle, "Pickler") == True; _ledger.append(1)
assert hasattr(pickle, "Unpickler") == True; _ledger.append(1)
assert hasattr(pickle, "PickleError") == True; _ledger.append(1)
assert hasattr(pickle, "PicklingError") == True; _ledger.append(1)
assert hasattr(pickle, "UnpicklingError") == True; _ledger.append(1)
assert hasattr(pickle, "HIGHEST_PROTOCOL") == True; _ledger.append(1)
assert hasattr(pickle, "DEFAULT_PROTOCOL") == True; _ledger.append(1)

# 2) pickle — value contracts (round-trip)
assert type(pickle.dumps(42)).__name__ == "bytes"; _ledger.append(1)
assert pickle.loads(pickle.dumps(42)) == 42; _ledger.append(1)
assert pickle.loads(pickle.dumps("hi")) == "hi"; _ledger.append(1)
assert pickle.loads(pickle.dumps([1, 2, 3])) == [1, 2, 3]; _ledger.append(1)
assert pickle.loads(pickle.dumps({"a": 1})) == {"a": 1}; _ledger.append(1)

# 3) numbers — hasattr core surface
assert hasattr(numbers, "Number") == True; _ledger.append(1)
assert hasattr(numbers, "Complex") == True; _ledger.append(1)
assert hasattr(numbers, "Real") == True; _ledger.append(1)
assert hasattr(numbers, "Rational") == True; _ledger.append(1)
assert hasattr(numbers, "Integral") == True; _ledger.append(1)

# 4) ipaddress — hasattr core surface
assert hasattr(ipaddress, "ip_address") == True; _ledger.append(1)
assert hasattr(ipaddress, "ip_network") == True; _ledger.append(1)
assert hasattr(ipaddress, "IPv4Address") == True; _ledger.append(1)
assert hasattr(ipaddress, "IPv6Address") == True; _ledger.append(1)
assert hasattr(ipaddress, "IPv4Network") == True; _ledger.append(1)
assert hasattr(ipaddress, "IPv6Network") == True; _ledger.append(1)
assert hasattr(ipaddress, "AddressValueError") == True; _ledger.append(1)
assert hasattr(ipaddress, "NetmaskValueError") == True; _ledger.append(1)

# 5) locale — hasattr core surface (conformant subset)
assert hasattr(locale, "getlocale") == True; _ledger.append(1)
assert hasattr(locale, "setlocale") == True; _ledger.append(1)
assert hasattr(locale, "LC_ALL") == True; _ledger.append(1)
assert hasattr(locale, "LC_CTYPE") == True; _ledger.append(1)
assert hasattr(locale, "LC_NUMERIC") == True; _ledger.append(1)
assert hasattr(locale, "LC_TIME") == True; _ledger.append(1)

# 6) gc — hasattr core surface (conformant subset)
assert hasattr(gc, "collect") == True; _ledger.append(1)
assert hasattr(gc, "disable") == True; _ledger.append(1)
assert hasattr(gc, "enable") == True; _ledger.append(1)
assert hasattr(gc, "isenabled") == True; _ledger.append(1)
assert hasattr(gc, "get_count") == True; _ledger.append(1)
assert hasattr(gc, "get_threshold") == True; _ledger.append(1)
assert hasattr(gc, "set_threshold") == True; _ledger.append(1)
assert hasattr(gc, "DEBUG_LEAK") == True; _ledger.append(1)

# 7) atexit — hasattr core surface
assert hasattr(atexit, "register") == True; _ledger.append(1)
assert hasattr(atexit, "unregister") == True; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_pickle_numbers_gc_atexit_value_ops {sum(_ledger)} asserts")
