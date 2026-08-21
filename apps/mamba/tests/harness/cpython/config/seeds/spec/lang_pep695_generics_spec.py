# lang_pep695_generics_spec.py — #3341 axis-1 PEP 695 generics FULL CPython
# contract.
#
# Companion to `pass/lang_pep695_generics.py` (working-subset regression
# sentinel). THIS file encodes the full CPython 3.12 PEP 695 runtime contract
# — including the parts mamba does NOT support today. Its parent directory
# `spec/` encodes the contract that this seed is expected to Fail today and
# serves as a permanent spec contract: when mamba implements any of the
# behaviors below, the runner detects the directory/outcome drift and surfaces
# the promotion as a one-line `git mv spec/<file> pass/<file>` edit.
#
# Each assertion is wrapped so unsupported-behavior surfaces (e.g. a
# TypeError raised by mamba on `Box[int]`) become explicit AssertionError
# failures and exit non-zero, rather than being silently swallowed.
#
# CPython 3.12 PEP 695 surface this seed asserts:
#   1. `Box[int](42)` — subscripted-class construction returns a valid
#      instance with the runtime attribute populated
#   2. `Box[int]` (bare subscript) returns a non-None alias-like value
#   3. `fn.__type_params__` is populated on a generic function
#   4. `Cls.__type_params__` is populated on a generic class
#   5. `Cls.__class_getitem__` is auto-generated on a generic class
#   6. `typing.TypeVar` interop alongside PEP 695 syntax
#
# Contract (encoded by parent directory `spec/`): expected to Fail today.

_ledger: list[int] = []

class _Box[T]:
    def __init__(self, v):
        self.v = v

# (1) Subscripted-class construction returns a usable instance
_b = None
_err1 = None
try:
    _b = _Box[int](42)
except Exception as e:
    _err1 = f"{type(e).__name__}: {e}"
assert _b is not None and _err1 is None and _b.v - 42 == 0, (
    f"_Box[int](42) constructs a real instance with .v == 42, "
    f"got _b={_b!r}, err={_err1!r}"
)
_ledger.append(1)

# (2) Bare subscript `_Box[int]` returns a non-None alias-like value
_alias = None
_err2 = None
try:
    _alias = _Box[int]
except Exception as e:
    _err2 = f"{type(e).__name__}: {e}"
assert _alias is not None and _err2 is None, (
    f"_Box[int] (bare subscript) returns a non-None alias-like, "
    f"got {_alias!r}, err={_err2!r}"
)
_ledger.append(1)

# (3) Generic function exposes __type_params__
def _fn[T](x: T) -> T:
    return x

_tp_fn = getattr(_fn, "__type_params__", None)
assert _tp_fn is not None and len(_tp_fn) >= 1, (
    f"_fn[T].__type_params__ is populated, got {_tp_fn!r}"
)
_ledger.append(1)

# (4) Generic class exposes __type_params__
class _Cls[T]:
    pass

_tp_cls = getattr(_Cls, "__type_params__", None)
assert _tp_cls is not None and len(_tp_cls) >= 1, (
    f"_Cls[T].__type_params__ is populated, got {_tp_cls!r}"
)
_ledger.append(1)

# (5) Generic class auto-generates __class_getitem__
assert hasattr(_Cls, "__class_getitem__"), (
    f"_Cls[T] auto-generates __class_getitem__, "
    f"got hasattr={hasattr(_Cls, '__class_getitem__')!r}"
)
_ledger.append(1)

# (6) typing.TypeVar interop alongside PEP 695 syntax
from typing import TypeVar
_T = TypeVar("_T")
assert _T is not None, (
    f"typing.TypeVar('_T') returns a non-None value alongside PEP 695, "
    f"got {_T!r}"
)
_ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_pep695_generics_spec {sum(_ledger)} asserts")
