# Operational AssertionPass divergence-spec fixture for the silent
# value-contract divergences of `hasattr(asyncio, 'Future')` (the
# documented "asyncio exposes the Future class" — mamba returns
# False — asyncio module is a dict), `hasattr(asyncio, 'Event')` (the
# documented "asyncio exposes the Event sync primitive" — mamba
# returns False), `hasattr(asyncio, 'Queue')` (the documented
# "asyncio exposes the async Queue" — mamba returns False),
# `hasattr(concurrent.futures, 'ThreadPoolExecutor')` (the documented
# "concurrent.futures exposes the ThreadPoolExecutor" — mamba returns
# False — concurrent.futures is None), `hasattr(concurrent.futures,
# 'Future')` (the documented "concurrent.futures exposes the Future
# class" — mamba returns False), `hasattr(multiprocessing, 'Pipe')`
# (the documented "multiprocessing exposes the Pipe factory" —
# mamba returns False), `hasattr(multiprocessing, 'Pool')` (the
# documented "multiprocessing exposes the Pool factory" — mamba
# returns False), `hasattr(mmap, 'mmap')` (the documented "mmap
# exposes the mmap class" — mamba returns False — mmap module is
# None), `hasattr(mmap, 'ACCESS_READ')` (the documented "mmap
# exposes the ACCESS_READ flag" — mamba returns False), and
# `mmap.ACCESS_READ == 1` (the documented "ACCESS_READ flag value
# is 1" — mamba returns None).
# Ten-pack pinned to atomic 283.
#
# Behavioral edges that CONFORM on mamba (asyncio — hasattr run/
# sleep/gather/wait_for/create_task/ensure_future. multiprocessing —
# hasattr Process/Queue/cpu_count/current_process + cpu_count > 0
# + int return type) are covered in the matching pass fixture
# `test_asyncio_concurrent_multiprocessing_mmap_value_ops`.
import asyncio
import concurrent.futures
import multiprocessing
import mmap


_ledger: list[int] = []

# 1) hasattr(asyncio, 'Future') — Future class
#    (mamba: returns False — asyncio is a dict)
assert hasattr(asyncio, "Future") == True; _ledger.append(1)

# 2) hasattr(asyncio, 'Event') — Event sync primitive
#    (mamba: returns False)
assert hasattr(asyncio, "Event") == True; _ledger.append(1)

# 3) hasattr(asyncio, 'Queue') — async Queue
#    (mamba: returns False)
assert hasattr(asyncio, "Queue") == True; _ledger.append(1)

# 4) hasattr(concurrent.futures, 'ThreadPoolExecutor')
#    (mamba: returns False — concurrent.futures is None)
assert hasattr(concurrent.futures, "ThreadPoolExecutor") == True; _ledger.append(1)

# 5) hasattr(concurrent.futures, 'Future') — Future class
#    (mamba: returns False)
assert hasattr(concurrent.futures, "Future") == True; _ledger.append(1)

# 6) hasattr(multiprocessing, 'Pipe') — Pipe factory
#    (mamba: returns False)
assert hasattr(multiprocessing, "Pipe") == True; _ledger.append(1)

# 7) hasattr(multiprocessing, 'Pool') — Pool factory
#    (mamba: returns False)
assert hasattr(multiprocessing, "Pool") == True; _ledger.append(1)

# 8) hasattr(mmap, 'mmap') — mmap class
#    (mamba: returns False — mmap module is None)
assert hasattr(mmap, "mmap") == True; _ledger.append(1)

# 9) hasattr(mmap, 'ACCESS_READ') — ACCESS_READ flag
#    (mamba: returns False)
assert hasattr(mmap, "ACCESS_READ") == True; _ledger.append(1)

# 10) mmap.ACCESS_READ == 1 — flag value
#     (mamba: returns None)
assert mmap.ACCESS_READ == 1; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_asyncio_concurrent_multiprocessing_mmap_silent {sum(_ledger)} asserts")
