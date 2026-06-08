# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cpython321_stdlib"
# dimension = "real_world"
# case = "test_logging_inspect_sys_keyword_value_ops"
# subject = "cpython321.test_logging_inspect_sys_keyword_value_ops"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/test_logging_inspect_sys_keyword_value_ops.py"
# status = "filled"
# ///
"""cpython321.test_logging_inspect_sys_keyword_value_ops: execute CPython 3.12 seed test_logging_inspect_sys_keyword_value_ops"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Atomic 273 pass conformance — logging module (hasattr DEBUG/INFO/
# WARNING/ERROR/CRITICAL/getLogger/basicConfig + DEBUG==10/INFO==20/
# WARNING==30/ERROR==40/CRITICAL==50) + inspect module (hasattr
# signature/getmembers/isclass/isfunction/ismethod + getargspec
# removed in 3.11 (both runtimes report False) + isclass(5)==False + isfunction(int)==False + isfunction(lambda)==
# True) + sys module (hasattr argv/path/modules/platform/version/
# version_info/exit/getrecursionlimit/maxsize/stdout/stderr/stdin/
# byteorder/getsizeof/settrace + argv/path are list, platform/version
# are str, maxsize>0, byteorder=='little') + keyword module (hasattr
# iskeyword/kwlist/softkwlist + iskeyword if/else/for/while/None/True
# True + iskeyword foo/bar False + kwlist is list + 'if' in kwlist +
# len(kwlist)==35).
# All asserts match between CPython 3.12 and mamba.
import logging
import inspect
import sys
import keyword


_ledger: list[int] = []

# 1) logging — hasattr level/factory surface
assert hasattr(logging, "DEBUG") == True; _ledger.append(1)
assert hasattr(logging, "INFO") == True; _ledger.append(1)
assert hasattr(logging, "WARNING") == True; _ledger.append(1)
assert hasattr(logging, "ERROR") == True; _ledger.append(1)
assert hasattr(logging, "CRITICAL") == True; _ledger.append(1)
assert hasattr(logging, "getLogger") == True; _ledger.append(1)
assert hasattr(logging, "basicConfig") == True; _ledger.append(1)

# 2) logging — level constants
assert logging.DEBUG == 10; _ledger.append(1)
assert logging.INFO == 20; _ledger.append(1)
assert logging.WARNING == 30; _ledger.append(1)
assert logging.ERROR == 40; _ledger.append(1)
assert logging.CRITICAL == 50; _ledger.append(1)

# 3) inspect — hasattr predicate surface
assert hasattr(inspect, "signature") == True; _ledger.append(1)
assert hasattr(inspect, "getmembers") == True; _ledger.append(1)
assert hasattr(inspect, "isclass") == True; _ledger.append(1)
assert hasattr(inspect, "isfunction") == True; _ledger.append(1)
assert hasattr(inspect, "ismethod") == True; _ledger.append(1)
assert hasattr(inspect, "getargspec") == False; _ledger.append(1)

# 4) inspect — predicate value contracts
assert inspect.isclass(5) == False; _ledger.append(1)
assert inspect.isfunction(int) == False; _ledger.append(1)
assert inspect.isfunction(lambda: 0) == True; _ledger.append(1)

# 5) sys — hasattr surface (config + io)
assert hasattr(sys, "argv") == True; _ledger.append(1)
assert hasattr(sys, "path") == True; _ledger.append(1)
assert hasattr(sys, "modules") == True; _ledger.append(1)
assert hasattr(sys, "platform") == True; _ledger.append(1)
assert hasattr(sys, "version") == True; _ledger.append(1)
assert hasattr(sys, "version_info") == True; _ledger.append(1)
assert hasattr(sys, "exit") == True; _ledger.append(1)
assert hasattr(sys, "getrecursionlimit") == True; _ledger.append(1)
assert hasattr(sys, "maxsize") == True; _ledger.append(1)
assert hasattr(sys, "stdout") == True; _ledger.append(1)
assert hasattr(sys, "stderr") == True; _ledger.append(1)
assert hasattr(sys, "stdin") == True; _ledger.append(1)
assert hasattr(sys, "byteorder") == True; _ledger.append(1)
assert hasattr(sys, "getsizeof") == True; _ledger.append(1)
assert hasattr(sys, "settrace") == True; _ledger.append(1)

# 6) sys — type/value contracts
assert isinstance(sys.argv, list) == True; _ledger.append(1)
assert isinstance(sys.path, list) == True; _ledger.append(1)
assert isinstance(sys.platform, str) == True; _ledger.append(1)
assert isinstance(sys.version, str) == True; _ledger.append(1)
assert (sys.maxsize > 0) == True; _ledger.append(1)
assert sys.byteorder == "little"; _ledger.append(1)

# 7) keyword — hasattr surface
assert hasattr(keyword, "iskeyword") == True; _ledger.append(1)
assert hasattr(keyword, "kwlist") == True; _ledger.append(1)
assert hasattr(keyword, "softkwlist") == True; _ledger.append(1)

# 8) keyword — iskeyword value contracts
assert keyword.iskeyword("if") == True; _ledger.append(1)
assert keyword.iskeyword("else") == True; _ledger.append(1)
assert keyword.iskeyword("for") == True; _ledger.append(1)
assert keyword.iskeyword("while") == True; _ledger.append(1)
assert keyword.iskeyword("None") == True; _ledger.append(1)
assert keyword.iskeyword("True") == True; _ledger.append(1)
assert keyword.iskeyword("foo") == False; _ledger.append(1)
assert keyword.iskeyword("bar") == False; _ledger.append(1)

# 9) keyword — kwlist contracts
assert isinstance(keyword.kwlist, list) == True; _ledger.append(1)
assert ("if" in keyword.kwlist) == True; _ledger.append(1)
assert len(keyword.kwlist) == 35; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_logging_inspect_sys_keyword_value_ops {sum(_ledger)} asserts")
