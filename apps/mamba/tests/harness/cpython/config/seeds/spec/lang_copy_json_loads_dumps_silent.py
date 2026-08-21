# Operational AssertionPass seed for SILENT divergences in `copy` and
# `json` — `json.loads` of bare-float literals, `json.dumps` of
# non-JSON-serializable types, default `ensure_ascii` escaping,
# `copy.copy` of immutable types (which CPython optimizes to "same
# object"), and `copy.deepcopy` preserving shared-reference identity.
# The matching subset (mutable-container roundtrip + non-float json
# roundtrip + indent/sort_keys) is covered by
# `test_copy_json_container_roundtrip_ops`; this fixture pins the
# CPython-only contracts that mamba currently elides.
#
# Surface (CPython AssertionPass; mamba diverges silently):
#   • json.loads("1.5") — should return float 1.5 (mamba returns the
#     int bit-pattern of the IEEE-754 double, e.g. 4613937818241073152
#     for -1.5);
#   • json.loads("1e2") — same: should return 100.0 (mamba returns the
#     bit pattern);
#   • json.dumps(<non-serializable>) — should raise TypeError (mamba
#     silently produces some output: function -> '16', set -> JSON
#     array, object() -> 'null');
#   • json.dumps with default `ensure_ascii=True` — should escape
#     non-ASCII chars (mamba returns the raw chars regardless);
#   • copy.copy(tuple) — CPython returns the SAME tuple object (mamba
#     creates a new one);
#   • copy.deepcopy preserving shared inner references — CPython keeps
#     `out[0] is out[1]` when input had it, mamba breaks the sharing.
import json
import copy
from typing import Any

_ledger: list[int] = []

# 1) json.loads of bare-float literal — returns float
_f1: Any = json.loads("1.5")
assert isinstance(_f1, float); _ledger.append(1)
assert _f1 == 1.5; _ledger.append(1)

_f2: Any = json.loads("-1.5")
assert isinstance(_f2, float); _ledger.append(1)
assert _f2 == -1.5; _ledger.append(1)

_f3: Any = json.loads("0.0")
assert isinstance(_f3, float); _ledger.append(1)
assert _f3 == 0.0; _ledger.append(1)

# Scientific notation also returns float
_f4: Any = json.loads("1e2")
assert isinstance(_f4, float); _ledger.append(1)
assert _f4 == 100.0; _ledger.append(1)

_f5: Any = json.loads("1.5e3")
assert isinstance(_f5, float); _ledger.append(1)
assert _f5 == 1500.0; _ledger.append(1)

# Round-trip via json.dumps preserves float
_back: Any = json.loads(json.dumps(3.14))
assert isinstance(_back, float); _ledger.append(1)
assert _back == 3.14; _ledger.append(1)

# 2) json.dumps of non-JSON types — CPython raises TypeError
def _dumps_fn():
    return json.dumps(lambda x: x)
try:
    _dumps_fn_out = _dumps_fn()
    _dumps_fn_result = "no-raise"
except TypeError:
    _dumps_fn_result = "typeerror"
except Exception:
    _dumps_fn_result = "other"
assert _dumps_fn_result == "typeerror"; _ledger.append(1)

# json.dumps(set) — TypeError
try:
    _dumps_set_out = json.dumps({1, 2, 3})
    _dumps_set_result = "no-raise"
except TypeError:
    _dumps_set_result = "typeerror"
except Exception:
    _dumps_set_result = "other"
assert _dumps_set_result == "typeerror"; _ledger.append(1)

# json.dumps(object()) — TypeError
try:
    _dumps_obj_out = json.dumps(object())
    _dumps_obj_result = "no-raise"
except TypeError:
    _dumps_obj_result = "typeerror"
except Exception:
    _dumps_obj_result = "other"
assert _dumps_obj_result == "typeerror"; _ledger.append(1)

# json.dumps(bytes) — TypeError (bytes is not JSON-encodable)
try:
    _dumps_bytes_out = json.dumps(b"hello")
    _dumps_bytes_result = "no-raise"
except TypeError:
    _dumps_bytes_result = "typeerror"
except Exception:
    _dumps_bytes_result = "other"
assert _dumps_bytes_result == "typeerror"; _ledger.append(1)

# 3) json.dumps default ensure_ascii=True — escapes non-ASCII
_esc: Any = json.dumps("café")
assert _esc == '"caf\\u00e9"'; _ledger.append(1)
# With explicit ensure_ascii=False, the raw character is preserved
_esc_off: Any = json.dumps("café", ensure_ascii=False)
assert _esc_off == '"café"'; _ledger.append(1)
# Higher code points
_emoji: Any = json.dumps("hello 你好")
assert "\\u" in _emoji; _ledger.append(1)

# 4) copy.copy on an immutable type returns the SAME object
_t: Any = (1, 2, 3)
assert copy.copy(_t) is _t; _ledger.append(1)
_s: Any = "immutable-string"
assert copy.copy(_s) is _s; _ledger.append(1)
_fs: Any = frozenset([1, 2, 3])
assert copy.copy(_fs) is _fs; _ledger.append(1)
# int / None / True — all share identity
_i: Any = 42
assert copy.copy(_i) is _i; _ledger.append(1)
assert copy.copy(None) is None; _ledger.append(1)
assert copy.copy(True) is True; _ledger.append(1)

# 5) deepcopy preserves shared-reference identity
_inner: Any = [10, 20]
_outer: Any = [_inner, _inner]
_dc: Any = copy.deepcopy(_outer)
# The inner lists were the SAME object before the deepcopy
assert _outer[0] is _outer[1]; _ledger.append(1)
# After deepcopy: the new outer still shares its two inner refs
assert _dc[0] is _dc[1]; _ledger.append(1)
# But the new inner is NOT the same as the old inner
assert _dc[0] is not _inner; _ledger.append(1)

# Cyclic data — deepcopy preserves the cycle
_cyc: Any = [1]
_cyc.append(_cyc)
_cyc_dc: Any = copy.deepcopy(_cyc)
# The cycle is preserved in the deep clone
assert _cyc_dc[1] is _cyc_dc; _ledger.append(1)

# 6) copy.copy on a user-defined object creates a NEW object
class _Holder:
    def __init__(self, v: int) -> None:
        self.v = v

_h: Any = _Holder(42)
_hc: Any = copy.copy(_h)
# New container, same field value
assert _hc is not _h; _ledger.append(1)
assert _hc.v == 42; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_copy_json_loads_dumps_silent {sum(_ledger)} asserts")
