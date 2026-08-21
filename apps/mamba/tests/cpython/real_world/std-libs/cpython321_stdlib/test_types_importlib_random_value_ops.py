# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cpython321_stdlib"
# dimension = "real_world"
# case = "test_types_importlib_random_value_ops"
# subject = "cpython321.test_types_importlib_random_value_ops"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/test_types_importlib_random_value_ops.py"
# status = "filled"
# ///
"""cpython321.test_types_importlib_random_value_ops: execute CPython 3.12 seed test_types_importlib_random_value_ops"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Atomic 312 pass conformance — types module (hasattr FunctionType/
# LambdaType/MethodType/ModuleType/GeneratorType/CoroutineType/Async
# GeneratorType/BuiltinFunctionType/BuiltinMethodType/CodeType/Frame
# Type/TracebackType/MappingProxyType/SimpleNamespace/DynamicClass
# Attribute/NoneType/EllipsisType/NotImplementedType/GenericAlias/
# UnionType + new_class/prepare_class/resolve_bases/coroutine) +
# importlib module (hasattr import_module/reload/invalidate_caches) +
# importlib.util module (hasattr find_spec) + linecache module
# (hasattr getline/getlines/checkcache/clearcache) + random module
# (hasattr choices/sample/shuffle/getrandbits/randint/randrange/random
# /uniform/seed/Random/triangular/betavariate/expovariate/gammavariate
# /gauss/lognormvariate/normalvariate/paretovariate/vonmisesvariate/
# weibullvariate + type(random.random()) float + type(random.randint(
# 1, 10)) int).
# All asserts match between CPython 3.12 and mamba.
import types
import importlib
from importlib import util as importlib_util
import linecache
import random


_ledger: list[int] = []

# 1) types — hasattr core surface (conformant subset)
assert hasattr(types, "FunctionType") == True; _ledger.append(1)
assert hasattr(types, "LambdaType") == True; _ledger.append(1)
assert hasattr(types, "MethodType") == True; _ledger.append(1)
assert hasattr(types, "ModuleType") == True; _ledger.append(1)
assert hasattr(types, "GeneratorType") == True; _ledger.append(1)
assert hasattr(types, "CoroutineType") == True; _ledger.append(1)
assert hasattr(types, "AsyncGeneratorType") == True; _ledger.append(1)
assert hasattr(types, "BuiltinFunctionType") == True; _ledger.append(1)
assert hasattr(types, "BuiltinMethodType") == True; _ledger.append(1)
assert hasattr(types, "CodeType") == True; _ledger.append(1)
assert hasattr(types, "FrameType") == True; _ledger.append(1)
assert hasattr(types, "TracebackType") == True; _ledger.append(1)
assert hasattr(types, "MappingProxyType") == True; _ledger.append(1)
assert hasattr(types, "SimpleNamespace") == True; _ledger.append(1)
assert hasattr(types, "DynamicClassAttribute") == True; _ledger.append(1)
assert hasattr(types, "NoneType") == True; _ledger.append(1)
assert hasattr(types, "EllipsisType") == True; _ledger.append(1)
assert hasattr(types, "NotImplementedType") == True; _ledger.append(1)
assert hasattr(types, "GenericAlias") == True; _ledger.append(1)
assert hasattr(types, "UnionType") == True; _ledger.append(1)
assert hasattr(types, "new_class") == True; _ledger.append(1)
assert hasattr(types, "prepare_class") == True; _ledger.append(1)
assert hasattr(types, "resolve_bases") == True; _ledger.append(1)
assert hasattr(types, "coroutine") == True; _ledger.append(1)

# 2) importlib — hasattr (conformant subset)
assert hasattr(importlib, "import_module") == True; _ledger.append(1)
assert hasattr(importlib, "reload") == True; _ledger.append(1)
assert hasattr(importlib, "invalidate_caches") == True; _ledger.append(1)

# 3) importlib.util — hasattr (conformant subset)
assert hasattr(importlib_util, "find_spec") == True; _ledger.append(1)

# 4) linecache — hasattr (conformant subset)
assert hasattr(linecache, "getline") == True; _ledger.append(1)
assert hasattr(linecache, "getlines") == True; _ledger.append(1)
assert hasattr(linecache, "checkcache") == True; _ledger.append(1)
assert hasattr(linecache, "clearcache") == True; _ledger.append(1)

# 5) random — hasattr core surface
assert hasattr(random, "choices") == True; _ledger.append(1)
assert hasattr(random, "sample") == True; _ledger.append(1)
assert hasattr(random, "shuffle") == True; _ledger.append(1)
assert hasattr(random, "getrandbits") == True; _ledger.append(1)
assert hasattr(random, "randint") == True; _ledger.append(1)
assert hasattr(random, "randrange") == True; _ledger.append(1)
assert hasattr(random, "random") == True; _ledger.append(1)
assert hasattr(random, "uniform") == True; _ledger.append(1)
assert hasattr(random, "seed") == True; _ledger.append(1)
assert hasattr(random, "Random") == True; _ledger.append(1)
assert hasattr(random, "triangular") == True; _ledger.append(1)
assert hasattr(random, "betavariate") == True; _ledger.append(1)
assert hasattr(random, "expovariate") == True; _ledger.append(1)
assert hasattr(random, "gammavariate") == True; _ledger.append(1)
assert hasattr(random, "gauss") == True; _ledger.append(1)
assert hasattr(random, "lognormvariate") == True; _ledger.append(1)
assert hasattr(random, "normalvariate") == True; _ledger.append(1)
assert hasattr(random, "paretovariate") == True; _ledger.append(1)
assert hasattr(random, "vonmisesvariate") == True; _ledger.append(1)
assert hasattr(random, "weibullvariate") == True; _ledger.append(1)

# 6) random — type contracts
assert type(random.random()).__name__ == "float"; _ledger.append(1)
assert type(random.randint(1, 10)).__name__ == "int"; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_types_importlib_random_value_ops {sum(_ledger)} asserts")
