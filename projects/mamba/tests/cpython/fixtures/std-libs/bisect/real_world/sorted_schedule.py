# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bisect"
# dimension = "real_world"
# case = "sorted_schedule"
# subject = "bisect.insort_right"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""bisect.insort_right: a task runner keeps a deadline-sorted schedule: insort_right inserts, bisect_left finds next-due"""
import bisect

# (deadline_seconds_since_epoch, task_id) pairs, pre-sorted by deadline.
schedule = [
    (100, "boot"),
    (200, "warmup"),
    (300, "first_request"),
    (500, "checkpoint"),
    (800, "shutdown"),
]

# Insert a new task at deadline=400 keeping the list sorted.
bisect.insort_right(schedule, (400, "midcycle_log"))
assert schedule == [
    (100, "boot"),
    (200, "warmup"),
    (300, "first_request"),
    (400, "midcycle_log"),
    (500, "checkpoint"),
    (800, "shutdown"),
], f"unexpected schedule after insort: {schedule!r}"

# Find the first task due at or after t=350. bisect_left on (350,) gives the
# leftmost slot where (350, ...) would fit -> index of the next task >= 350.
idx = bisect.bisect_left(schedule, (350,))
assert idx == 3, f"expected idx=3, got {idx}"
assert schedule[idx] == (400, "midcycle_log"), f"unexpected next-due: {schedule[idx]!r}"

# bisect is an alias for bisect_right.
assert bisect.bisect(schedule, (300,)) == bisect.bisect_right(schedule, (300,))

# insort is an alias for insort_right.
schedule2 = list(schedule)
bisect.insort(schedule2, (450, "snapshot"))
bisect.insort_right(schedule, (450, "snapshot"))
assert schedule == schedule2, "insort alias diverged from insort_right"

print("sorted_schedule OK")
