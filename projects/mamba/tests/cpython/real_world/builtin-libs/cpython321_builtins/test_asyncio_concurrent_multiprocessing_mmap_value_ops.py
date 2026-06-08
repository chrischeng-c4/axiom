# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "cpython321_builtins"
# dimension = "real_world"
# case = "test_asyncio_concurrent_multiprocessing_mmap_value_ops"
# subject = "cpython321.test_asyncio_concurrent_multiprocessing_mmap_value_ops"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/test_asyncio_concurrent_multiprocessing_mmap_value_ops.py"
# status = "filled"
# ///
"""cpython321.test_asyncio_concurrent_multiprocessing_mmap_value_ops: execute CPython 3.12 seed test_asyncio_concurrent_multiprocessing_mmap_value_ops"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Atomic 283 pass conformance — asyncio module (hasattr run/sleep/
# gather/wait_for/create_task/ensure_future) + multiprocessing module
# (hasattr Process/Queue/cpu_count/current_process + cpu_count > 0 +
# cpu_count returns int).
# All asserts match between CPython 3.12 and mamba.
import asyncio
import multiprocessing


_ledger: list[int] = []

# 1) asyncio — hasattr scheduling primitives
assert hasattr(asyncio, "run") == True; _ledger.append(1)
assert hasattr(asyncio, "sleep") == True; _ledger.append(1)
assert hasattr(asyncio, "gather") == True; _ledger.append(1)
assert hasattr(asyncio, "wait_for") == True; _ledger.append(1)
assert hasattr(asyncio, "create_task") == True; _ledger.append(1)
assert hasattr(asyncio, "ensure_future") == True; _ledger.append(1)

# 2) multiprocessing — hasattr top-level worker surface
assert hasattr(multiprocessing, "Process") == True; _ledger.append(1)
assert hasattr(multiprocessing, "Queue") == True; _ledger.append(1)
assert hasattr(multiprocessing, "cpu_count") == True; _ledger.append(1)
assert hasattr(multiprocessing, "current_process") == True; _ledger.append(1)

# 3) multiprocessing — cpu_count value contracts
assert (multiprocessing.cpu_count() > 0) == True; _ledger.append(1)
assert isinstance(multiprocessing.cpu_count(), int) == True; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_asyncio_concurrent_multiprocessing_mmap_value_ops {sum(_ledger)} asserts")
