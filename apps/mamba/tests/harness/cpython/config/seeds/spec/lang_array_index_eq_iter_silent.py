# Operational AssertionPass seed for SILENT divergences in
# `array.array` introspection + comparison + iteration + boundary
# validation. The matching subset (tolist-based round trips, mutators,
# read-only typecode/itemsize, count/index) is covered by
# `test_array_typed_storage_ops`; this fixture pins the CPython-only
# contracts that mamba currently elides.
#
# Surface (CPython AssertionPass; mamba diverges silently):
#   • len(array) — should return the element count (mamba returns 0);
#   • array[i] — should return the element at index i (mamba returns
#     None);
#   • iter(array) — should iterate the elements (mamba raises
#     TypeError "object is not iterable");
#   • array == array — element-wise equality (mamba returns False even
#     for two arrays with identical content);
#   • array.array("q") / ("Q") — should preserve the "q"/"Q" typecode
#     identity (mamba normalizes to "l"/"L"; itemsize is still 8 so the
#     storage width is correct, only the typecode label differs);
#   • Invalid typecode — `array.array("Z", [1])` should raise
#     ValueError (mamba silently constructs an int-handle);
#   • Out-of-range value for signed/unsigned typecodes — should raise
#     OverflowError (mamba silently constructs an int-handle).
import array
from typing import Any

_ledger: list[int] = []

# 1) len(array) — CPython spec: element count
_a4: Any = array.array("i", [10, 20, 30, 40])
assert len(_a4) == 4; _ledger.append(1)
_a0: Any = array.array("i")
assert len(_a0) == 0; _ledger.append(1)
_a1: Any = array.array("i", [99])
assert len(_a1) == 1; _ledger.append(1)
_ab: Any = array.array("b", [1, 2, 3, 4, 5])
assert len(_ab) == 5; _ledger.append(1)
_af: Any = array.array("f", [1.5, 2.5, 3.5])
assert len(_af) == 3; _ledger.append(1)

# 2) array[i] — CPython spec: element at index
_ax: Any = array.array("i", [10, 20, 30, 40])
assert _ax[0] == 10; _ledger.append(1)
assert _ax[1] == 20; _ledger.append(1)
assert _ax[2] == 30; _ledger.append(1)
assert _ax[3] == 40; _ledger.append(1)
# Negative index — counts from the end
assert _ax[-1] == 40; _ledger.append(1)
assert _ax[-2] == 30; _ledger.append(1)

# 3) iter(array) — CPython spec: iterates the elements
_ai: Any = array.array("i", [1, 2, 3])
_iter_collected = []
for _e in iter(_ai):
    _iter_collected.append(_e)
assert _iter_collected == [1, 2, 3]; _ledger.append(1)
# list(array) — bulk realize
assert list(array.array("i", [10, 20, 30])) == [10, 20, 30]; _ledger.append(1)
# sum(array)
assert sum(array.array("i", [1, 2, 3, 4])) == 10; _ledger.append(1)
# max(array)
assert max(array.array("i", [5, 1, 9, 3])) == 9; _ledger.append(1)
# min(array)
assert min(array.array("i", [5, 1, 9, 3])) == 1; _ledger.append(1)

# 4) array == array — CPython spec: element-wise comparison
_eq1: Any = array.array("i", [1, 2, 3])
_eq2: Any = array.array("i", [1, 2, 3])
assert _eq1 == _eq2; _ledger.append(1)
# Different content — not equal
_ne1: Any = array.array("i", [1, 2, 3])
_ne2: Any = array.array("i", [1, 2, 4])
assert _ne1 != _ne2; _ledger.append(1)
# Different length — not equal
_ne3: Any = array.array("i", [1, 2])
_ne4: Any = array.array("i", [1, 2, 3])
assert _ne3 != _ne4; _ledger.append(1)
# Empty arrays of same typecode — equal
_e_a: Any = array.array("i")
_e_b: Any = array.array("i")
assert _e_a == _e_b; _ledger.append(1)

# 5) "q"/"Q" typecode-identity — CPython spec preserves the typecode
_aq: Any = array.array("q", [1, 2, 3])
assert _aq.typecode == "q"; _ledger.append(1)
_aQ: Any = array.array("Q", [1, 2, 3])
assert _aQ.typecode == "Q"; _ledger.append(1)

# 6) Invalid typecode — CPython spec raises ValueError
_invalid: Any = None
try:
    _invalid = array.array("Z", [1])
    _invalid_outcome = "no-raise"
except ValueError:
    _invalid_outcome = "valueerror"
except Exception:
    _invalid_outcome = "other"
assert _invalid_outcome == "valueerror"; _ledger.append(1)

# 7) Out-of-range value — CPython spec raises OverflowError
#    signed byte "b" range = [-128, 127]
_ov_b: Any = None
try:
    _ov_b = array.array("b", [128])
    _ov_b_outcome = "no-raise"
except OverflowError:
    _ov_b_outcome = "overflowerror"
except Exception:
    _ov_b_outcome = "other"
assert _ov_b_outcome == "overflowerror"; _ledger.append(1)

#    unsigned byte "B" range = [0, 255]
_ov_B: Any = None
try:
    _ov_B = array.array("B", [-1])
    _ov_B_outcome = "no-raise"
except OverflowError:
    _ov_B_outcome = "overflowerror"
except Exception:
    _ov_B_outcome = "other"
assert _ov_B_outcome == "overflowerror"; _ledger.append(1)

#    Bad value type — non-numeric for numeric typecode
_bad_type: Any = None
try:
    _bad_type = array.array("i", ["string"])
    _bad_type_outcome = "no-raise"
except TypeError:
    _bad_type_outcome = "typeerror"
except Exception:
    _bad_type_outcome = "other"
assert _bad_type_outcome == "typeerror"; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_array_index_eq_iter_silent {sum(_ledger)} asserts")
