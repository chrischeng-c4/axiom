# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "cpython321_builtins"
# dimension = "real_world"
# case = "test_pickle_struct_itertools_dis_heapq_random_enum_unique_value_ops"
# subject = "cpython321.test_pickle_struct_itertools_dis_heapq_random_enum_unique_value_ops"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/test_pickle_struct_itertools_dis_heapq_random_enum_unique_value_ops.py"
# status = "filled"
# ///
"""cpython321.test_pickle_struct_itertools_dis_heapq_random_enum_unique_value_ops: execute CPython 3.12 seed test_pickle_struct_itertools_dis_heapq_random_enum_unique_value_ops"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Atomic 244 pass conformance — pickle round-trip / struct partial / itertools
# accumulate sum+lambda / functools surface / dis partial surface / inspect
# signature hasattr / heapq nlargest/nsmallest basic / random non-seeded
# membership/range / enum.unique class binding that match between
# CPython 3.12 and mamba.
import pickle
import struct
import itertools as it
import functools
import dis
import inspect
import heapq
import random
import enum


_ledger: list[int] = []

# 1) pickle protocol constant + round-trip value ops
assert pickle.HIGHEST_PROTOCOL == 5; _ledger.append(1)
assert pickle.loads(pickle.dumps(42)) == 42; _ledger.append(1)
assert pickle.loads(pickle.dumps("hello")) == "hello"; _ledger.append(1)
assert pickle.loads(pickle.dumps([1, 2, 3])) == [1, 2, 3]; _ledger.append(1)
assert pickle.loads(pickle.dumps({"a": 1, "b": [2, 3]})) == {"a": 1, "b": [2, 3]}; _ledger.append(1)
assert pickle.loads(pickle.dumps((1, "a", 3.14))) == (1, "a", 3.14); _ledger.append(1)

# 2) struct pack/unpack/calcsize/iter_unpack
assert struct.pack("3i", 1, 2, 3) == b"\x01\x00\x00\x00\x02\x00\x00\x00\x03\x00\x00\x00"; _ledger.append(1)
assert struct.unpack("3i", struct.pack("3i", 1, 2, 3)) == (1, 2, 3); _ledger.append(1)
assert struct.calcsize("3i") == 12; _ledger.append(1)
assert struct.calcsize("2d") == 16; _ledger.append(1)
assert list(struct.iter_unpack("i", struct.pack("3i", 10, 20, 30))) == [(10,), (20,), (30,)]; _ledger.append(1)

# 3) itertools accumulate — default sum + lambda mul
assert list(it.accumulate([1, 2, 3, 4])) == [1, 3, 6, 10]; _ledger.append(1)
assert list(it.accumulate([1, 2, 3, 4], lambda a, b: a * b)) == [1, 2, 6, 24]; _ledger.append(1)

# 4) functools hasattr surface
assert hasattr(functools, "singledispatchmethod") == True; _ledger.append(1)
assert hasattr(functools, "partial") == True; _ledger.append(1)
assert hasattr(functools, "cmp_to_key") == True; _ledger.append(1)

# 5) dis partial surface
assert hasattr(dis, "dis") == True; _ledger.append(1)
assert hasattr(dis, "Instruction") == True; _ledger.append(1)
assert hasattr(dis, "get_instructions") == True; _ledger.append(1)
assert hasattr(dis, "opname") == True; _ledger.append(1)
assert hasattr(dis, "opmap") == True; _ledger.append(1)
assert callable(dis.get_instructions) == True; _ledger.append(1)

# 6) inspect.signature hasattr (deep surface covered in spec)
assert hasattr(inspect, "signature") == True; _ledger.append(1)
assert hasattr(inspect, "isclass") == True; _ledger.append(1)
assert hasattr(inspect, "ismethod") == True; _ledger.append(1)
assert hasattr(inspect, "getmembers") == True; _ledger.append(1)

# 7) heapq nlargest/nsmallest basic (no key arg)
assert heapq.nlargest(3, [5, 1, 4, 9, 2, 6, 3]) == [9, 6, 5]; _ledger.append(1)
assert heapq.nsmallest(3, [5, 1, 4, 9, 2, 6, 3]) == [1, 2, 3]; _ledger.append(1)

# 8) random non-seeded — membership/range value contracts
assert random.choice([1, 2, 3, 4, 5]) in [1, 2, 3, 4, 5]; _ledger.append(1)
assert 1 <= random.randint(1, 100) <= 100; _ledger.append(1)
assert 10 <= random.randrange(10, 20) < 20; _ledger.append(1)
assert 0.0 <= random.random() < 1.0; _ledger.append(1)
assert len(random.sample(range(10), 3)) == 3; _ledger.append(1)
_lst = [1, 2, 3, 4, 5]
random.shuffle(_lst)
assert sorted(_lst) == [1, 2, 3, 4, 5]; _ledger.append(1)

# 9) enum.unique class binding
assert hasattr(enum, "unique") == True; _ledger.append(1)
assert hasattr(enum, "Enum") == True; _ledger.append(1)
assert hasattr(enum, "IntEnum") == True; _ledger.append(1)
assert hasattr(enum, "Flag") == True; _ledger.append(1)
assert hasattr(enum, "IntFlag") == True; _ledger.append(1)
assert hasattr(enum, "auto") == True; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_pickle_struct_itertools_dis_heapq_random_enum_unique_value_ops {sum(_ledger)} asserts")
