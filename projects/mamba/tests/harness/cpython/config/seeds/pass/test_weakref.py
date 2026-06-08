# test_weakref.py — #3440 axis-1 stdlib weakref AssertionPass seed.
#
# Mamba-authored seed exercising the `weakref` module surface called
# out in the issue:
#   ref + deref None after gc, WeakValueDictionary auto-removal,
#   finalize callback.
#
# Surface coverage (asserts run at module scope; no helper closures per
# the mamba top-level def() quirk in test_math.py):
#   1. Module identity + public surface (hasattr).
#   2. weakref.ref(obj) — deref while alive yields obj; weakref.ref
#      class identity.
#   3. Callback fires on collection: weakref.ref(obj, callback) — after
#      del + gc.collect() the callback is invoked exactly once.
#   4. Deref-None after gc — r() returns None when the referent is
#      collected.
#   5. WeakValueDictionary — set + retrieve while alive; auto-removal
#      after referent collected.
#   6. WeakKeyDictionary — set + retrieve while key alive.
#   7. WeakSet — membership while alive; len.
#   8. weakref.finalize — `.alive` transitions True→False after
#      collection and the registered callback runs.
#   9. weakref.proxy — attribute access proxies to the underlying object.
#
# Mamba note: this seed depends on gc.collect() actually freeing
# unreachable user objects so the weak-ref callbacks fire. If mamba's
# GC does not collect on demand, the relevant asserts will trip; that
# is the runtime gap the issue is tracking.
#
# Contract with `cpython_lib_test_runner.rs`:
#   - Each `assert` runs at top level. Inverting any assert raises
#     AssertionError → non-zero exit → runner classifies as `Fail`.
#   - After every assertion has executed, the seed emits
#     `MAMBA_ASSERTION_PASS: test_weakref N asserts` to stdout.

import gc
import weakref

_ledger: list[int] = []


# Module-level helper classes — must be non-built-in so weakref accepts
# instances. (built-in tuples/lists/strs are unhashable for weakref.)
class _Holder:
    """Plain class — weakref-friendly."""


class _HolderH:
    """Weakref + hashable holder for dict/set keys."""

    def __init__(self, tag: str) -> None:
        self.tag = tag

    def __hash__(self) -> int:
        return hash(self.tag)

    def __eq__(self, other: object) -> bool:
        return isinstance(other, _HolderH) and self.tag == other.tag


_callback_log: list[str] = []


def _on_collect(_ref):  # type: ignore[no-untyped-def]
    _callback_log.append("collected")


_finalize_log: list[int] = []


def _on_finalize() -> None:
    _finalize_log.append(1)


# 1. Module identity + public surface.
assert weakref.__name__ == "weakref", "weakref.__name__"
_ledger.append(1)
assert hasattr(weakref, "ref"), "exposes ref"
_ledger.append(1)
assert hasattr(weakref, "proxy"), "exposes proxy"
_ledger.append(1)
assert hasattr(weakref, "WeakValueDictionary"), "exposes WeakValueDictionary"
_ledger.append(1)
assert hasattr(weakref, "WeakKeyDictionary"), "exposes WeakKeyDictionary"
_ledger.append(1)
assert hasattr(weakref, "WeakSet"), "exposes WeakSet"
_ledger.append(1)
assert hasattr(weakref, "finalize"), "exposes finalize"
_ledger.append(1)

# 2. weakref.ref — deref while alive.
_obj = _Holder()
_r = weakref.ref(_obj)
assert _r() is _obj, "ref() returns the referent while it is alive"
_ledger.append(1)
assert isinstance(_r, weakref.ref), "weakref.ref(obj) returns a ref instance"
_ledger.append(1)

# 3. Callback registration — fires on collection.
_obj_cb = _Holder()
_r_cb = weakref.ref(_obj_cb, _on_collect)
assert _r_cb() is _obj_cb, "ref-with-callback also derefs while alive"
_ledger.append(1)
assert _callback_log == [], "callback has NOT fired before referent collected"
_ledger.append(1)
del _obj_cb
gc.collect()
assert _callback_log == ["collected"], (
    "callback fires exactly once after del + gc.collect()"
)
_ledger.append(1)

# 4. Deref-None after gc.
_obj_d = _Holder()
_r_d = weakref.ref(_obj_d)
assert _r_d() is _obj_d, "weak ref derefs to live referent"
_ledger.append(1)
del _obj_d
gc.collect()
assert _r_d() is None, "weak ref derefs to None after referent collected"
_ledger.append(1)

# 5. WeakValueDictionary — set, get, auto-removal on collection.
_wvd: "weakref.WeakValueDictionary[str, _Holder]" = weakref.WeakValueDictionary()
_v = _Holder()
_wvd["k"] = _v
assert _wvd["k"] is _v, "WeakValueDictionary['k'] returns the stored value while alive"
_ledger.append(1)
assert "k" in _wvd, "key membership while value alive"
_ledger.append(1)
assert len(_wvd) - 1 == 0, "len(WeakValueDictionary) == 1 while value alive"
_ledger.append(1)
del _v
gc.collect()
assert "k" not in _wvd, "key removed from WeakValueDictionary after value collected"
_ledger.append(1)
assert len(_wvd) == 0, "WeakValueDictionary empty after only-value collected"
_ledger.append(1)

# 6. WeakKeyDictionary — set / get while key alive.
_wkd: "weakref.WeakKeyDictionary[_HolderH, str]" = weakref.WeakKeyDictionary()
_k = _HolderH("alpha")
_wkd[_k] = "value-alpha"
assert _wkd[_k] == "value-alpha", "WeakKeyDictionary read by live key"
_ledger.append(1)
assert _k in _wkd, "key membership while key alive"
_ledger.append(1)
assert len(_wkd) - 1 == 0, "len(WeakKeyDictionary) == 1 while key alive"
_ledger.append(1)

# 7. WeakSet — membership while alive; len.
_ws: "weakref.WeakSet[_HolderH]" = weakref.WeakSet()
_member = _HolderH("beta")
_ws.add(_member)
assert _member in _ws, "WeakSet membership while element alive"
_ledger.append(1)
assert len(_ws) - 1 == 0, "len(WeakSet) == 1 after add"
_ledger.append(1)

# 8. weakref.finalize — alive transitions + callback runs on collection.
_obj_f = _Holder()
_fin = weakref.finalize(_obj_f, _on_finalize)
assert _fin.alive == True, "finalize.alive True while target alive"
_ledger.append(1)
assert _finalize_log == [], "finalize callback has NOT fired yet"
_ledger.append(1)
del _obj_f
gc.collect()
assert _finalize_log == [1], (
    "finalize callback runs exactly once after target collected"
)
_ledger.append(1)
assert _fin.alive == False, "finalize.alive False after target collected"
_ledger.append(1)

# 9. weakref.proxy — attribute access pass-through.
_proxy_target = _Holder()
_proxy_target.payload = "hello"  # type: ignore[attr-defined]
_p = weakref.proxy(_proxy_target)
assert _p.payload == "hello", "proxy forwards attribute access to live referent"
_ledger.append(1)

# Emit the proof-of-execution marker. Per `ASSERTION_PASS_MARKERS` in
# `cpython_lib_test_runner.rs`, presence of this token escalates the
# outcome from ImportPass to AssertionPass.
print(f"MAMBA_ASSERTION_PASS: test_weakref {len(_ledger)} asserts")
