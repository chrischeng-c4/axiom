# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tracemalloc"
# dimension = "behavior"
# case = "statistic_str_formatting"
# subject = "tracemalloc.Snapshot"
# kind = "semantic"
# xfail = "mamba does not implement the tracemalloc.Snapshot class (#1261 long-tail stub batch)"
# mem_carveout = ""
# source = "Lib/test/test_tracemalloc.py"
# status = "filled"
# ///
"""tracemalloc.Snapshot: str() of a Statistic and a StatisticDiff render the human-readable size/count/average summary strings"""
import tracemalloc

RAW1 = [
    (0, 10, (("a.py", 2), ("b.py", 4)), 3),
    (0, 10, (("a.py", 2), ("b.py", 4)), 3),
    (0, 10, (("a.py", 2), ("b.py", 4)), 3),
    (1, 2, (("a.py", 5), ("b.py", 4)), 3),
    (2, 66, (("b.py", 1),), 1),
    (3, 7, (("<unknown>", 0),), 1),
]
RAW2 = [
    (0, 10, (("a.py", 2), ("b.py", 4)), 3),
    (2, 2, (("a.py", 5), ("b.py", 4)), 3),
    (2, 5000, (("a.py", 5), ("b.py", 4)), 3),
    (4, 400, (("c.py", 578),), 1),
]
snap1 = tracemalloc.Snapshot(RAW1, 2)
snap2 = tracemalloc.Snapshot(RAW2, 2)

top = snap1.statistics("lineno")[0]
grew = snap2.compare_to(snap1, "lineno")[0]

# str() formatting of Statistic and StatisticDiff.
assert str(top) == "b.py:1: size=66 B, count=1, average=66 B", "Statistic str"
assert (
    str(grew)
    == "a.py:5: size=5002 B (+5000 B), count=2 (+1), average=2501 B"
), "StatisticDiff str"

print("statistic_str_formatting OK")
