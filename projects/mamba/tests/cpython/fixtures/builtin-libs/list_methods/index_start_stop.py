# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
# `list.index(value, start=0, stop=len)` and `tuple.index(...)` ignored
# their start/stop arguments — `mb_list_index` / `mb_tuple_index` always
# scanned from index 0. So:
#
#   [10, 20, 30, 20, 10].index(20, 2)       # → 1   (wrong; should be 3)
#   [10, 20, 30, 20, 10].index(10, 1, 5)    # → 0   (wrong; should be 4)
#
# Fix in `runtime/list_ops.rs` + `runtime/tuple_ops.rs`:
#   - Add `mb_list_index_range(list, value, start, stop)` and the
#     symmetric `mb_tuple_index_range`. Both clamp `start` / `stop` the
#     CPython way: defaults `0` / `len`, negatives counted from the end
#     and floored at 0, positives capped at `len`.
#   - Dispatchers grow an `argc()` helper and route 1-, 2-, 3-arg
#     forms through the new `_range` functions.
#
# `tuple.index` keeps its existing `-1` not-found sentinel for now —
# converting that to `raise ValueError` is a separate fix tracked
# elsewhere (the test at `tuple_ops.rs:566` still expects `-1`).

L = [10, 20, 30, 20, 10]

# Single-arg form unchanged.
print(L.index(10))                    # 0
print(L.index(20))                    # 1
print(L.index(30))                    # 2

# With explicit start.
print(L.index(20, 2))                 # 3   (skip the first 20)
print(L.index(10, 1))                 # 4
print(L.index(10, -3))                # 4   (negative start)

# With start + stop.
print(L.index(10, 1, 5))              # 4
print(L.index(20, 0, 3))              # 1
print(L.index(20, 0, 100))            # 1   (stop clamps to len)
print(L.index(10, -100, 100))         # 0   (start clamps to 0)

# Not found in slice → ValueError.
try:
    L.index(99)
except ValueError as e:
    print("VE:", e)
try:
    L.index(20, 4)
except ValueError as e:
    print("VE-bound:", e)
try:
    L.index(20, 0, 1)
except ValueError as e:
    print("VE-stop:", e)

# Empty / inverted bounds.
try:
    L.index(10, 5, 5)
except ValueError as e:
    print("VE-empty:", e)
try:
    L.index(10, 3, 1)                 # stop < start → no scan
except ValueError as e:
    print("VE-inverted:", e)

# Tuple index — same shape, same start/stop clamping.
t = (1, 2, 3, 2, 1)
print(t.index(2))                     # 1
print(t.index(2, 2))                  # 3
print(t.index(1, 2, 5))               # 4
print(t.index(1, -2))                 # 4