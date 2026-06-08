# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "cpython321_core_lang"
# dimension = "real_world"
# case = "lang_statistics_asyncio_mp_silent"
# subject = "cpython321.lang_statistics_asyncio_mp_silent"
# kind = "semantic"
# xfail = "CPython 3.12 seed spec; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/spec/lang_statistics_asyncio_mp_silent.py"
# status = "filled"
# ///
"""cpython321.lang_statistics_asyncio_mp_silent: execute CPython 3.12 seed lang_statistics_asyncio_mp_silent"""
# mamba-xfail: CPython 3.12 seed spec; mamba promotion pending
# Operational AssertionPass divergence-spec fixture for the silent
# value-contract divergences of `statistics.mean([1,2,3]) == 2.0` (the
# documented "arithmetic mean of [1,2,3] is the float 2.0" — mamba
# returns the i64 bit pattern 4611686018427387904 — float-as-i64),
# `statistics.fmean([1,2,3]) == 2.0` (the documented "floating-point
# mean of [1,2,3] is 2.0" — mamba returns the i64 bit pattern —
# float-as-i64), `statistics.variance([1,2,3,4,5]) == 2.5` (the
# documented "sample variance of [1..5] is 2.5" — mamba returns the
# i64 bit pattern — float-as-i64), `hasattr(asyncio, 'Future')` (the
# documented "asyncio exposes the Future class" — mamba returns False),
# `hasattr(asyncio, 'Lock')` (the documented "asyncio exposes the Lock
# synchronization primitive" — mamba returns False), `hasattr(asyncio,
# 'CancelledError')` (the documented "asyncio exposes the
# CancelledError exception" — mamba returns False), `hasattr(asyncio,
# 'iscoroutine')` (the documented "asyncio exposes the iscoroutine
# predicate" — mamba returns False), `hasattr(multiprocessing, 'Pool')`
# (the documented "multiprocessing exposes the Pool class" — mamba
# returns False), `hasattr(multiprocessing, 'Manager')` (the documented
# "multiprocessing exposes the Manager factory" — mamba returns False),
# and `hasattr(pstats, 'StatsProfile')` (the documented "pstats exposes
# the StatsProfile dataclass" — mamba returns False).
# Ten-pack pinned to atomic 314.
#
# Behavioral edges that CONFORM on mamba (statistics — hasattr mean/
# median/mode/stdev/variance/median_low/median_high/median_grouped/
# pstdev/pvariance/fmean/geometric_mean/harmonic_mean/multimode/
# quantiles/StatisticsError/NormalDist/correlation/covariance/linear_
# regression + statistics.mode([1,1,2]) == 1 + statistics.median_low(
# [1,2,3,4]) == 2 + statistics.median_high([1,2,3,4]) == 3 + statistics
# .median([1,2,3]) == 2. asyncio — hasattr run/sleep/gather/wait/wait_
# for/ensure_future/create_task. multiprocessing — hasattr Process/
# Queue/current_process/cpu_count. mailbox — hasattr Mailbox/Maildir/
# mbox/MH/Babyl/MMDF/Message/MaildirMessage/mboxMessage/Error/NoSuch
# MailboxError. chunk/sunau/audioop — hasattr Chunk/Au_read/Au_write/
# open/Error + mul/add/avg/cross/error. venv/ensurepip — hasattr Env
# Builder/create/main + bootstrap/version. pdb/profile/cProfile/pstats
# — hasattr Pdb/set_trace/post_mortem/run/runeval/runcall/pm + Profile
# /run/runctx + Profile/run/runctx + Stats/SortKey. compileall/module
# finder — hasattr compile_file/compile_dir/compile_path + ModuleFinder
# /Module/ReplacePackage. wsgiref subpackages — hasattr make_server/
# WSGIServer/WSGIRequestHandler + setup_testing_defaults/FileWrapper +
# Headers + BaseHandler/SimpleHandler) are covered in the matching
# pass fixture `test_statistics_asyncio_mp_value_ops`.
import statistics
import asyncio
import multiprocessing
import pstats


_ledger: list[int] = []

# 1) statistics.mean([1,2,3]) == 2.0 — arithmetic mean of [1,2,3]
#    (mamba: returns the i64 bit pattern 4611686018427387904 — float-as-i64)
assert statistics.mean([1, 2, 3]) == 2.0; _ledger.append(1)

# 2) statistics.fmean([1,2,3]) == 2.0 — floating-point mean of [1,2,3]
#    (mamba: returns the i64 bit pattern — float-as-i64)
assert statistics.fmean([1, 2, 3]) == 2.0; _ledger.append(1)

# 3) statistics.variance([1,2,3,4,5]) == 2.5 — sample variance of [1..5]
#    (mamba: returns the i64 bit pattern — float-as-i64)
assert statistics.variance([1, 2, 3, 4, 5]) == 2.5; _ledger.append(1)

# 4) hasattr(asyncio, 'Future') — Future class
#    (mamba: returns False)
assert hasattr(asyncio, "Future") == True; _ledger.append(1)

# 5) hasattr(asyncio, 'Lock') — Lock synchronization primitive
#    (mamba: returns False)
assert hasattr(asyncio, "Lock") == True; _ledger.append(1)

# 6) hasattr(asyncio, 'CancelledError') — CancelledError exception
#    (mamba: returns False)
assert hasattr(asyncio, "CancelledError") == True; _ledger.append(1)

# 7) hasattr(asyncio, 'iscoroutine') — iscoroutine predicate
#    (mamba: returns False)
assert hasattr(asyncio, "iscoroutine") == True; _ledger.append(1)

# 8) hasattr(multiprocessing, 'Pool') — Pool class
#    (mamba: returns False)
assert hasattr(multiprocessing, "Pool") == True; _ledger.append(1)

# 9) hasattr(multiprocessing, 'Manager') — Manager factory
#    (mamba: returns False)
assert hasattr(multiprocessing, "Manager") == True; _ledger.append(1)

# 10) hasattr(pstats, 'StatsProfile') — StatsProfile dataclass
#     (mamba: returns False)
assert hasattr(pstats, "StatsProfile") == True; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_statistics_asyncio_mp_silent {sum(_ledger)} asserts")
