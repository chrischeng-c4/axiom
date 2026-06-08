# Atomic 227 pass conformance — heapq/bisect/queue/threading/tempfile/shutil/
# glob/enum/collections/itertools hasattr + value ops that match between
# CPython 3.12 and mamba.
import heapq
import bisect
import queue
import threading
import tempfile
import shutil
import glob
import enum
import collections
import itertools

_ledger: list[int] = []

# 1) heapq — value ops
xs = [3, 1, 4, 1, 5, 9, 2, 6]
heap = list(xs)
heapq.heapify(heap)
assert heap[0] == 1; _ledger.append(1)
heapq.heappush(heap, 0)
assert heap[0] == 0; _ledger.append(1)
_v = heapq.heappop(heap)
assert _v == 0; _ledger.append(1)
assert heapq.nlargest(3, xs) == [9, 6, 5]; _ledger.append(1)
assert heapq.nsmallest(3, xs) == [1, 1, 2]; _ledger.append(1)
assert list(heapq.merge([1, 3, 5], [2, 4, 6])) == [1, 2, 3, 4, 5, 6]; _ledger.append(1)

# 2) bisect — value ops
xs = [1, 3, 5, 7, 9]
assert bisect.bisect_left(xs, 5) == 2; _ledger.append(1)
assert bisect.bisect_right(xs, 5) == 3; _ledger.append(1)
assert bisect.bisect(xs, 5) == 3; _ledger.append(1)
assert bisect.bisect_left(xs, 4) == 2; _ledger.append(1)
assert bisect.bisect_right(xs, 4) == 2; _ledger.append(1)
ys = [1, 3, 5, 7, 9]
bisect.insort(ys, 4)
assert ys == [1, 3, 4, 5, 7, 9]; _ledger.append(1)
ys2 = [1, 3, 5, 7, 9]
bisect.insort_left(ys2, 4)
assert ys2 == [1, 3, 4, 5, 7, 9]; _ledger.append(1)

# 3) queue — FIFO and LIFO ordering
q = queue.Queue()
q.put(1)
q.put(2)
q.put(3)
assert q.qsize() == 3; _ledger.append(1)
assert q.get() == 1; _ledger.append(1)
assert q.get() == 2; _ledger.append(1)
assert q.empty() == False; _ledger.append(1)
assert q.get() == 3; _ledger.append(1)
assert q.empty() == True; _ledger.append(1)

lq = queue.LifoQueue()
lq.put(1)
lq.put(2)
assert lq.get() == 2; _ledger.append(1)
assert lq.get() == 1; _ledger.append(1)

# 4) threading — full hasattr surface
assert hasattr(threading, "Thread") == True; _ledger.append(1)
assert hasattr(threading, "Lock") == True; _ledger.append(1)
assert hasattr(threading, "RLock") == True; _ledger.append(1)
assert hasattr(threading, "Event") == True; _ledger.append(1)
assert hasattr(threading, "Condition") == True; _ledger.append(1)
assert hasattr(threading, "Semaphore") == True; _ledger.append(1)
assert hasattr(threading, "BoundedSemaphore") == True; _ledger.append(1)
assert hasattr(threading, "Barrier") == True; _ledger.append(1)
assert hasattr(threading, "Timer") == True; _ledger.append(1)
assert hasattr(threading, "local") == True; _ledger.append(1)
assert hasattr(threading, "active_count") == True; _ledger.append(1)
assert hasattr(threading, "current_thread") == True; _ledger.append(1)
assert hasattr(threading, "main_thread") == True; _ledger.append(1)
assert hasattr(threading, "enumerate") == True; _ledger.append(1)
assert hasattr(threading, "get_ident") == True; _ledger.append(1)
assert hasattr(threading, "get_native_id") == True; _ledger.append(1)
assert hasattr(threading, "settrace") == True; _ledger.append(1)
assert hasattr(threading, "setprofile") == True; _ledger.append(1)
assert hasattr(threading, "stack_size") == True; _ledger.append(1)
assert hasattr(threading, "TIMEOUT_MAX") == True; _ledger.append(1)
assert hasattr(threading, "ExceptHookArgs") == True; _ledger.append(1)
assert hasattr(threading, "excepthook") == True; _ledger.append(1)

# 5) tempfile — full hasattr surface
assert hasattr(tempfile, "TemporaryFile") == True; _ledger.append(1)
assert hasattr(tempfile, "NamedTemporaryFile") == True; _ledger.append(1)
assert hasattr(tempfile, "TemporaryDirectory") == True; _ledger.append(1)
assert hasattr(tempfile, "SpooledTemporaryFile") == True; _ledger.append(1)
assert hasattr(tempfile, "mkstemp") == True; _ledger.append(1)
assert hasattr(tempfile, "mkdtemp") == True; _ledger.append(1)
assert hasattr(tempfile, "mktemp") == True; _ledger.append(1)
assert hasattr(tempfile, "gettempdir") == True; _ledger.append(1)
assert hasattr(tempfile, "gettempprefix") == True; _ledger.append(1)
assert hasattr(tempfile, "tempdir") == True; _ledger.append(1)

# 6) shutil — full hasattr surface
assert hasattr(shutil, "copy") == True; _ledger.append(1)
assert hasattr(shutil, "copy2") == True; _ledger.append(1)
assert hasattr(shutil, "copyfile") == True; _ledger.append(1)
assert hasattr(shutil, "copyfileobj") == True; _ledger.append(1)
assert hasattr(shutil, "copymode") == True; _ledger.append(1)
assert hasattr(shutil, "copystat") == True; _ledger.append(1)
assert hasattr(shutil, "copytree") == True; _ledger.append(1)
assert hasattr(shutil, "rmtree") == True; _ledger.append(1)
assert hasattr(shutil, "move") == True; _ledger.append(1)
assert hasattr(shutil, "disk_usage") == True; _ledger.append(1)
assert hasattr(shutil, "chown") == True; _ledger.append(1)
assert hasattr(shutil, "which") == True; _ledger.append(1)
assert hasattr(shutil, "make_archive") == True; _ledger.append(1)
assert hasattr(shutil, "unpack_archive") == True; _ledger.append(1)
assert hasattr(shutil, "get_terminal_size") == True; _ledger.append(1)
assert hasattr(shutil, "SameFileError") == True; _ledger.append(1)

# 7) glob — full hasattr surface
assert hasattr(glob, "glob") == True; _ledger.append(1)
assert hasattr(glob, "iglob") == True; _ledger.append(1)
assert hasattr(glob, "escape") == True; _ledger.append(1)
assert hasattr(glob, "has_magic") == True; _ledger.append(1)

# 8) enum — top-level hasattr surface
assert hasattr(enum, "Enum") == True; _ledger.append(1)
assert hasattr(enum, "IntEnum") == True; _ledger.append(1)
assert hasattr(enum, "Flag") == True; _ledger.append(1)
assert hasattr(enum, "IntFlag") == True; _ledger.append(1)
assert hasattr(enum, "StrEnum") == True; _ledger.append(1)
assert hasattr(enum, "auto") == True; _ledger.append(1)
assert hasattr(enum, "unique") == True; _ledger.append(1)
assert hasattr(enum, "EnumType") == True; _ledger.append(1)

# 9) collections — full hasattr surface
assert hasattr(collections, "OrderedDict") == True; _ledger.append(1)
assert hasattr(collections, "defaultdict") == True; _ledger.append(1)
assert hasattr(collections, "Counter") == True; _ledger.append(1)
assert hasattr(collections, "deque") == True; _ledger.append(1)
assert hasattr(collections, "namedtuple") == True; _ledger.append(1)
assert hasattr(collections, "ChainMap") == True; _ledger.append(1)
assert hasattr(collections, "UserDict") == True; _ledger.append(1)
assert hasattr(collections, "UserList") == True; _ledger.append(1)
assert hasattr(collections, "UserString") == True; _ledger.append(1)

# 10) itertools — full hasattr surface
assert hasattr(itertools, "count") == True; _ledger.append(1)
assert hasattr(itertools, "cycle") == True; _ledger.append(1)
assert hasattr(itertools, "repeat") == True; _ledger.append(1)
assert hasattr(itertools, "accumulate") == True; _ledger.append(1)
assert hasattr(itertools, "chain") == True; _ledger.append(1)
assert hasattr(itertools, "compress") == True; _ledger.append(1)
assert hasattr(itertools, "dropwhile") == True; _ledger.append(1)
assert hasattr(itertools, "takewhile") == True; _ledger.append(1)
assert hasattr(itertools, "filterfalse") == True; _ledger.append(1)
assert hasattr(itertools, "groupby") == True; _ledger.append(1)
assert hasattr(itertools, "islice") == True; _ledger.append(1)
assert hasattr(itertools, "starmap") == True; _ledger.append(1)
assert hasattr(itertools, "tee") == True; _ledger.append(1)
assert hasattr(itertools, "zip_longest") == True; _ledger.append(1)
assert hasattr(itertools, "product") == True; _ledger.append(1)
assert hasattr(itertools, "permutations") == True; _ledger.append(1)
assert hasattr(itertools, "combinations") == True; _ledger.append(1)
assert hasattr(itertools, "combinations_with_replacement") == True; _ledger.append(1)
assert hasattr(itertools, "pairwise") == True; _ledger.append(1)
assert hasattr(itertools, "batched") == True; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_heapq_bisect_queue_threading_tempfile_value_ops {sum(_ledger)} asserts")
