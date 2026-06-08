# Operational AssertionPass divergence-spec fixture for the
# silent value-contract divergence of the
# `queue` / `io` / `array` / `enum` / `dataclasses`
# five-pack pinned to atomic 227:
# `queue` (the documented `PriorityQueue.get()` priority
# ordering — mamba's `PriorityQueue` returns items in insertion
# order, not priority order), `io` (the documented
# `StringIO/BytesIO.getvalue()` accumulation contract — mamba's
# `getvalue()` silently returns an empty value after writes),
# `array` (the documented `array.array("i", [...])` typed
# sequence contract — mamba's `array.array(...)` collapses to
# an empty handle: `[0]` returns None and `len()` returns 0),
# `enum` (the documented `hasattr(enum, "EnumMeta") == True`
# legacy metaclass alias — mamba exposes only the newer
# `EnumType` name), and `dataclasses` (the documented extended
# `hasattr(dataclasses, "make_dataclass") / "replace" /
# "is_dataclass" / "MISSING" / "KW_ONLY" /
# "FrozenInstanceError" / "Field" == True` extended hasattr
# surface — mamba exposes only the basic
# `dataclass/field/fields/asdict/astuple` core).
#
# Behavioral edges that CONFORM on mamba (heapq heapify/push/
# pop/nlargest/nsmallest/merge, bisect bisect_left/right/insort,
# queue.Queue + queue.LifoQueue FIFO/LIFO ordering, threading
# top-level hasattr surface, tempfile/shutil/glob/collections/
# itertools top-level hasattr surface, enum top-level
# Enum/IntEnum/Flag/IntFlag/StrEnum/auto/unique/EnumType/Member
# hasattr surface) are covered in the matching pass fixture
# `test_heapq_bisect_queue_threading_tempfile_value_ops`.
from typing import Any
import queue as _queue_mod
import io as _io_mod
import array as _array_mod
import enum as _enum_mod
import dataclasses as _dataclasses_mod

queue: Any = _queue_mod
io: Any = _io_mod
array: Any = _array_mod
enum: Any = _enum_mod
dataclasses: Any = _dataclasses_mod


_ledger: list[int] = []

# 1) queue.PriorityQueue — priority ordering contract
#    (mamba: returns insertion-order item `(3, "c")` instead of
#    lowest-priority `(1, "a")`)
_pq = queue.PriorityQueue()
_pq.put((3, "c"))
_pq.put((1, "a"))
_pq.put((2, "b"))
assert _pq.get() == (1, "a"); _ledger.append(1)

# 2) io.StringIO — `getvalue()` accumulation contract
#    (mamba: silently returns empty string after writes)
_sbuf = io.StringIO()
_sbuf.write("hello")
_sbuf.write(" world")
assert _sbuf.getvalue() == "hello world"; _ledger.append(1)

# 3) io.BytesIO — `getvalue()` accumulation contract
#    (mamba: silently returns empty bytes after writes)
_bbuf = io.BytesIO()
_bbuf.write(b"hello")
_bbuf.write(b" world")
assert _bbuf.getvalue() == b"hello world"; _ledger.append(1)

# 4) array.array — typed-sequence value contract
#    (mamba: array.array(...) collapses to empty handle —
#    [0] returns None, len() returns 0)
_a = array.array("i", [1, 2, 3, 4])
assert _a[0] == 1; _ledger.append(1)
assert len(_a) == 4; _ledger.append(1)

# 5) enum — `EnumMeta` legacy metaclass alias hasattr
#    (mamba: only the newer `EnumType` name is exposed)
assert hasattr(enum, "EnumMeta") == True; _ledger.append(1)

# 6) dataclasses — extended module hasattr surface
#    (mamba: make_dataclass / replace / is_dataclass / MISSING /
#    KW_ONLY / FrozenInstanceError / Field all False)
assert hasattr(dataclasses, "make_dataclass") == True; _ledger.append(1)
assert hasattr(dataclasses, "replace") == True; _ledger.append(1)
assert hasattr(dataclasses, "is_dataclass") == True; _ledger.append(1)
assert hasattr(dataclasses, "MISSING") == True; _ledger.append(1)
assert hasattr(dataclasses, "KW_ONLY") == True; _ledger.append(1)
assert hasattr(dataclasses, "FrozenInstanceError") == True; _ledger.append(1)
assert hasattr(dataclasses, "Field") == True; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_queue_io_array_enum_dataclasses_silent {sum(_ledger)} asserts")
