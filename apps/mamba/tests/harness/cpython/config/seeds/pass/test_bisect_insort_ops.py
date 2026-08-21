# Operational AssertionPass seed for bisect insort surfaces beyond
# test_bisect_ops (which covers bisect_left / bisect_right basics).
# Surface: insort_left and insort_right both insert into a sorted
# list while keeping it sorted; insort_left places the new value
# BEFORE any existing equal entries, insort_right places it AFTER;
# bisect_left / bisect_right report the corresponding insertion
# indices for duplicates and for empty lists; insort on an empty
# list initializes it with the single element; repeated insort
# preserves global ordering regardless of insertion order; insort
# at the head (smaller than all) and tail (larger than all) extends
# the list correctly; bisect.bisect is an alias for bisect_right.
import bisect
_ledger: list[int] = []

# insort_left into the middle of a sorted list
a = [1, 3, 5, 7]
bisect.insort_left(a, 4)
assert a == [1, 3, 4, 5, 7]; _ledger.append(1)

# insort_right into the middle — same result when no duplicates
b = [1, 3, 5, 7]
bisect.insort_right(b, 4)
assert b == [1, 3, 4, 5, 7]; _ledger.append(1)

# insort_left with a duplicate — places BEFORE the existing equal run
c = [1, 2, 2, 3]
bisect.insort_left(c, 2)
assert c == [1, 2, 2, 2, 3]; _ledger.append(1)
# (left + right produce equal lists, but the index where the new
# element ends up differs — exercised below via bisect_*)

# insort_right with a duplicate — places AFTER the existing equal run
d = [1, 2, 2, 3]
bisect.insort_right(d, 2)
assert d == [1, 2, 2, 2, 3]; _ledger.append(1)

# bisect_left / bisect_right on duplicates — the index they would
# insert at; left returns the FIRST equal position, right returns
# ONE PAST the last equal position
assert bisect.bisect_left([1, 2, 2, 3], 2) == 1; _ledger.append(1)
assert bisect.bisect_right([1, 2, 2, 3], 2) == 3; _ledger.append(1)

# Empty-list edge cases — both bisect_left and bisect_right return 0
assert bisect.bisect_left([], 5) == 0; _ledger.append(1)
assert bisect.bisect_right([], 5) == 0; _ledger.append(1)

# insort into an empty list — initializes it with the single element
e: list[int] = []
bisect.insort(e, 5)
assert e == [5]; _ledger.append(1)

# Repeated insort preserves ordering regardless of insert order
f: list[int] = []
bisect.insort(f, 3)
bisect.insort(f, 1)
bisect.insort(f, 2)
assert f == [1, 2, 3]; _ledger.append(1)

# insort at the head — value smaller than all existing entries
g = [5, 6, 7]
bisect.insort(g, 1)
assert g == [1, 5, 6, 7]; _ledger.append(1)

# insort at the tail — value larger than all existing entries
h = [1, 2, 3]
bisect.insort(h, 10)
assert h == [1, 2, 3, 10]; _ledger.append(1)

# bisect.bisect is an alias for bisect_right
assert bisect.bisect([1, 2, 3, 4], 2) == 2; _ledger.append(1)
assert bisect.bisect([1, 2, 2, 3], 2) == 3; _ledger.append(1)

# Long run of duplicates — left vs right indices differ by the run length
runs = [1, 5, 5, 5, 5, 9]
assert bisect.bisect_left(runs, 5) == 1; _ledger.append(1)
assert bisect.bisect_right(runs, 5) == 5; _ledger.append(1)

# Value below the minimum — index 0
assert bisect.bisect_left([10, 20, 30], 5) == 0; _ledger.append(1)
assert bisect.bisect_right([10, 20, 30], 5) == 0; _ledger.append(1)

# Value above the maximum — index == len
assert bisect.bisect_left([10, 20, 30], 99) == 3; _ledger.append(1)
assert bisect.bisect_right([10, 20, 30], 99) == 3; _ledger.append(1)

# A series of insorts builds up a sorted list from a shuffled input
shuffled = [3, 1, 4, 1, 5, 9, 2, 6, 5]
out: list[int] = []
for x in shuffled:
    bisect.insort(out, x)
assert out == [1, 1, 2, 3, 4, 5, 5, 6, 9]; _ledger.append(1)
print(f"MAMBA_ASSERTION_PASS: test_bisect_insort_ops {sum(_ledger)} asserts")
