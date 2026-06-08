# Operational AssertionPass seed for SILENT divergences across the
# typing parametrized-generic + Any sentinel surface +
# @dataclass auto-init + dataclasses extended module helpers +
# contextlib extended class identifiers + contextlib.suppress
# behavior + inspect extended module helpers + inspect.signature
# parameter extraction + inspect.isfunction value-truth + abc.ABC
# abstract-method enforcement pinned by atomic 180: `typing` (the
# documented `List[T]` / `Dict[K, V]` / `Optional[T]` /
# `Iterable` / `Sequence` / `Mapping` surface), `dataclasses`
# (the documented `is_dataclass` / `replace` / `make_dataclass`
# / `MISSING` hasattr surface + the documented `@dataclass`
# auto-`__init__` field-binding contract), `contextlib` (the
# documented `ExitStack` / `closing` / `AbstractContextManager`
# class identifiers + the documented `suppress(*excs)` exception-
# suppression behavior), `inspect` (the documented `Signature` /
# `Parameter` / `ismodule` / `getdoc` / `getsource` /
# `getfullargspec` hasattr surface + the documented
# `inspect.signature(fn).parameters` extraction contract + the
# documented `inspect.isfunction(fn)` value-truth contract),
# and `abc` (the documented abstract-method instantiation
# enforcement on the abstract base class).
#
# The matching subset (typing module hasattr layer (List /
# Dict / Set / Tuple / Optional / Union / Any / Callable /
# TypeVar / Generic / Protocol / ClassVar / Type / Iterator),
# partial dataclasses module hasattr surface (dataclass /
# field / fields / asdict / astuple), abc module hasattr
# surface + concrete-subclass instance dispatch, partial
# contextlib module hasattr surface (contextmanager /
# suppress / nullcontext) + nullcontext value-passthrough,
# partial inspect module hasattr surface (signature /
# isfunction / isclass / ismethod / getmembers)) is covered
# by `test_typing_dataclasses_abc_contextlib_inspect_hasattr_
# ops`; this fixture pins the CPython-only contracts that
# mamba currently elides.
#
# Surface (CPython AssertionPass; mamba diverges silently):
#   • typing.List[int] is not None — documented parametrized
#     generic (mamba: returns None — the parametrized-generic
#     surface is broken);
#   • typing.Dict[str, int] is not None — documented
#     parametrized generic (mamba: returns None);
#   • typing.Optional[int] is not None — documented
#     parametrized generic (mamba: returns None);
#   • hasattr(typing, "Iterable") is True — documented
#     class identifier (mamba: False);
#   • hasattr(typing, "Sequence") is True — documented
#     class identifier (mamba: False);
#   • hasattr(typing, "Mapping") is True — documented
#     class identifier (mamba: False);
#   • hasattr(dataclasses, "is_dataclass") is True —
#     documented helper (mamba: False);
#   • hasattr(dataclasses, "replace") is True — documented
#     helper (mamba: False);
#   • hasattr(dataclasses, "make_dataclass") is True —
#     documented helper (mamba: False);
#   • hasattr(dataclasses, "MISSING") is True — documented
#     sentinel (mamba: False);
#   • @dataclass; Point(1, 2).x == 1 — documented auto-init
#     field-binding contract (mamba: returns None — the
#     auto-generated __init__ is missing or broken);
#   • hasattr(contextlib, "ExitStack") is True — documented
#     class identifier (mamba: False);
#   • hasattr(contextlib, "closing") is True — documented
#     helper (mamba: False);
#   • hasattr(contextlib, "AbstractContextManager") is True
#     — documented class identifier (mamba: False);
#   • contextlib.suppress(ValueError) actually suppresses
#     the matching ValueError — documented behavior
#     (mamba: the suppress context manager does not
#     suppress the exception);
#   • hasattr(inspect, "Signature") is True — documented
#     class identifier (mamba: False);
#   • hasattr(inspect, "Parameter") is True — documented
#     class identifier (mamba: False);
#   • hasattr(inspect, "ismodule") is True — documented
#     helper (mamba: False);
#   • hasattr(inspect, "getdoc") is True — documented
#     helper (mamba: False);
#   • hasattr(inspect, "getsource") is True — documented
#     helper (mamba: False);
#   • hasattr(inspect, "getfullargspec") is True —
#     documented helper (mamba: False);
#   • list(inspect.signature(fn).parameters.keys()) ==
#     ["a", "b", "c"] — documented parameter-extraction
#     contract (mamba: returns the empty signature ());
#   • inspect.isfunction(fn) is True — documented value-
#     truth contract on a defined function (mamba: False);
#   • abc.ABC instantiation enforcement — instantiating an
#     abstract base class raises TypeError (mamba: the
#     instantiation succeeds silently).
import typing as _typing_mod
import dataclasses as _dataclasses_mod
import contextlib as _contextlib_mod
import inspect as _inspect_mod
import abc as _abc_mod
from typing import Any

# Module bindings retyped as `Any` to bypass Pyright stub-driven
# narrowing — every spec contract below probes documented public
# generics / class identifiers / helpers / abstract-enforcement
# behavior that mamba's bundled type stubs do not surface
# accurately.
typing: Any = _typing_mod
dataclasses: Any = _dataclasses_mod
contextlib: Any = _contextlib_mod
inspect: Any = _inspect_mod
abc: Any = _abc_mod


@dataclasses.dataclass
class _Point:
    x: int
    y: int


class _Shape(abc.ABC):
    @abc.abstractmethod
    def area(self):
        pass


def _my_fn(a, b, c=10):
    return a + b + c


_ledger: list[int] = []

# 1) typing — parametrized generics
assert typing.List[int] is not None; _ledger.append(1)
assert typing.Dict[str, int] is not None; _ledger.append(1)
assert typing.Optional[int] is not None; _ledger.append(1)

# 2) typing — extended class identifiers
assert hasattr(typing, "Iterable") == True; _ledger.append(1)
assert hasattr(typing, "Sequence") == True; _ledger.append(1)
assert hasattr(typing, "Mapping") == True; _ledger.append(1)

# 3) dataclasses — extended module helpers
assert hasattr(dataclasses, "is_dataclass") == True; _ledger.append(1)
assert hasattr(dataclasses, "replace") == True; _ledger.append(1)
assert hasattr(dataclasses, "make_dataclass") == True; _ledger.append(1)
assert hasattr(dataclasses, "MISSING") == True; _ledger.append(1)

# 4) @dataclass — auto-init field binding
_p = _Point(1, 2)
assert _p.x == 1; _ledger.append(1)
assert _p.y == 2; _ledger.append(1)

# 5) contextlib — extended class identifiers
assert hasattr(contextlib, "ExitStack") == True; _ledger.append(1)
assert hasattr(contextlib, "closing") == True; _ledger.append(1)
assert hasattr(contextlib, "AbstractContextManager") == True; _ledger.append(1)

# 6) contextlib.suppress — exception-suppression behavior
_suppressed = False
try:
    with contextlib.suppress(ValueError):
        raise ValueError("ignored")
    _suppressed = True
except ValueError:
    _suppressed = False
assert _suppressed == True; _ledger.append(1)

# 7) inspect — extended module helpers
assert hasattr(inspect, "Signature") == True; _ledger.append(1)
assert hasattr(inspect, "Parameter") == True; _ledger.append(1)
assert hasattr(inspect, "ismodule") == True; _ledger.append(1)
assert hasattr(inspect, "getdoc") == True; _ledger.append(1)
assert hasattr(inspect, "getsource") == True; _ledger.append(1)
assert hasattr(inspect, "getfullargspec") == True; _ledger.append(1)

# 8) inspect.signature — parameter-extraction contract
_sig = inspect.signature(_my_fn)
assert list(_sig.parameters.keys()) == ["a", "b", "c"]; _ledger.append(1)

# 9) inspect.isfunction — value-truth on defined function
assert inspect.isfunction(_my_fn) == True; _ledger.append(1)

# 10) abc.ABC — abstract instantiation enforcement
_raised = False
try:
    _abc_inst = _Shape()
    _raised = False
except TypeError:
    _raised = True
assert _raised == True; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_typing_dataclass_inspect_suppress_silent {sum(_ledger)} asserts")
