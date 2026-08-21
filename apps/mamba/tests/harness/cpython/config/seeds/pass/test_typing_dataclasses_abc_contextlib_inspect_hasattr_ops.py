# Operational AssertionPass seed for the value contract of the
# `typing` / `dataclasses` / `abc` / `contextlib` / `inspect`
# five-pack pinned to atomic 180: `typing` (the documented
# generic-class identifier hasattr surface — `List` / `Dict` /
# `Set` / `Tuple` / `Optional` / `Union` / `Any` / `Callable`
# / `TypeVar` / `Generic` / `Protocol` / `ClassVar` / `Type` /
# `Iterator`), `dataclasses` (the documented `dataclass` /
# `field` / `fields` / `asdict` / `astuple` module-level
# helper hasattr surface), `abc` (the full documented `ABC` /
# `ABCMeta` / `abstractmethod` / `abstractproperty` /
# `abstractclassmethod` / `abstractstaticmethod` module-level
# helper hasattr surface + the concrete-subclass instance
# method dispatch contract), `contextlib` (the documented
# `contextmanager` / `suppress` / `nullcontext` module-level
# helper hasattr surface + the documented `nullcontext(value)`
# value-passthrough contract), and `inspect` (the documented
# `signature` / `isfunction` / `isclass` / `ismethod` /
# `getmembers` module-level helper hasattr surface).
#
# The matching subset between mamba and CPython is the full
# `typing` generic-class identifier hasattr layer (`List` /
# `Dict` / `Set` / `Tuple` / `Optional` / `Union` / `Any` /
# `Callable` / `TypeVar` / `Generic` / `Protocol` /
# `ClassVar` / `Type` / `Iterator` — `Iterable` / `Sequence`
# / `Mapping` DIVERGE + the parametrized-generic instance
# layer DIVERGES + the Any sentinel-identity layer DIVERGES),
# the partial `dataclasses` module hasattr surface
# (`dataclass` / `field` / `fields` / `asdict` / `astuple` —
# `is_dataclass` / `replace` / `make_dataclass` / `MISSING`
# DIVERGE + the @dataclass auto-init layer DIVERGES), the
# full `abc` module hasattr surface + the concrete-subclass
# instance method dispatch layer (the abstract-method
# enforcement on the abstract base class instantiation
# DIVERGES), the partial `contextlib` module hasattr
# surface (`contextmanager` / `suppress` / `nullcontext` —
# `ExitStack` / `closing` / `AbstractContextManager`
# DIVERGE + the `suppress` behavior + the `contextmanager`
# yield-value layer DIVERGE) + the `nullcontext(value)`
# value-passthrough layer, and the partial `inspect`
# module hasattr surface (`signature` / `isfunction` /
# `isclass` / `ismethod` / `getmembers` — `Signature` /
# `Parameter` / `ismodule` / `getdoc` / `getsource` /
# `getfullargspec` DIVERGE + the inspect.signature
# parameter-extraction layer + isfunction value-truth
# layer DIVERGE).
#
# Surface in this fixture:
#   • typing — module hasattr surface (List / Dict / Set /
#     Tuple / Optional / Union / Any / Callable / TypeVar /
#     Generic / Protocol / ClassVar / Type / Iterator);
#   • dataclasses — partial module hasattr surface
#     (dataclass / field / fields / asdict / astuple);
#   • abc — module hasattr surface (ABC / ABCMeta /
#     abstractmethod / abstractproperty / abstractclassmethod
#     / abstractstaticmethod);
#   • abc — concrete-subclass instance method dispatch
#     (Square.area() returns side**2);
#   • contextlib — partial module hasattr surface
#     (contextmanager / suppress / nullcontext);
#   • contextlib.nullcontext — value-passthrough contract;
#   • inspect — partial module hasattr surface (signature /
#     isfunction / isclass / ismethod / getmembers).
#
# Behavioral edges that DIVERGE on mamba (typing.List[int] /
# Dict / Set / Tuple / Optional / Union / TypeVar / Callable
# all return None — parametrized-generic instance broken,
# typing.Any returns a lambda not the documented Any
# sentinel, hasattr(typing, "Iterable") / "Sequence" /
# "Mapping" False, hasattr(dataclasses, "is_dataclass") /
# "replace" / "make_dataclass" / "MISSING" False,
# @dataclass Point(1, 2).x returns None — auto-init layer
# broken, hasattr(contextlib, "ExitStack") / "closing" /
# "AbstractContextManager" False, contextlib.suppress
# fails to suppress the matching exception,
# @contextlib.contextmanager yields integer 1 not the
# documented value, hasattr(inspect, "Signature") /
# "Parameter" / "ismodule" / "getdoc" / "getsource" /
# "getfullargspec" False, inspect.signature(fn) returns
# "()" not "(a, b, c=10)", inspect.isfunction(fn) returns
# False on a defined function, abc.ABC abstract enforcement
# is silent — Shape() succeeds where it should raise
# TypeError) are covered in the matching spec fixture
# `lang_typing_dataclass_inspect_suppress_silent`.
import typing
import dataclasses
import abc
import contextlib
import inspect


class _Shape(abc.ABC):
    @abc.abstractmethod
    def area(self):
        pass


class _Square(_Shape):
    def __init__(self, side):
        self.side = side

    def area(self):
        return self.side * self.side


_ledger: list[int] = []

# 1) typing — module hasattr surface (Iterable / Sequence /
#    Mapping DIVERGE — moved to spec fixture)
assert hasattr(typing, "List") == True; _ledger.append(1)
assert hasattr(typing, "Dict") == True; _ledger.append(1)
assert hasattr(typing, "Set") == True; _ledger.append(1)
assert hasattr(typing, "Tuple") == True; _ledger.append(1)
assert hasattr(typing, "Optional") == True; _ledger.append(1)
assert hasattr(typing, "Union") == True; _ledger.append(1)
assert hasattr(typing, "Any") == True; _ledger.append(1)
assert hasattr(typing, "Callable") == True; _ledger.append(1)
assert hasattr(typing, "TypeVar") == True; _ledger.append(1)
assert hasattr(typing, "Generic") == True; _ledger.append(1)
assert hasattr(typing, "Protocol") == True; _ledger.append(1)
assert hasattr(typing, "ClassVar") == True; _ledger.append(1)
assert hasattr(typing, "Type") == True; _ledger.append(1)
assert hasattr(typing, "Iterator") == True; _ledger.append(1)

# 2) dataclasses — partial module hasattr surface
#    (is_dataclass / replace / make_dataclass / MISSING
#    DIVERGE — moved to spec fixture)
assert hasattr(dataclasses, "dataclass") == True; _ledger.append(1)
assert hasattr(dataclasses, "field") == True; _ledger.append(1)
assert hasattr(dataclasses, "fields") == True; _ledger.append(1)
assert hasattr(dataclasses, "asdict") == True; _ledger.append(1)
assert hasattr(dataclasses, "astuple") == True; _ledger.append(1)

# 3) abc — module hasattr surface
assert hasattr(abc, "ABC") == True; _ledger.append(1)
assert hasattr(abc, "ABCMeta") == True; _ledger.append(1)
assert hasattr(abc, "abstractmethod") == True; _ledger.append(1)
assert hasattr(abc, "abstractproperty") == True; _ledger.append(1)
assert hasattr(abc, "abstractclassmethod") == True; _ledger.append(1)
assert hasattr(abc, "abstractstaticmethod") == True; _ledger.append(1)

# 4) abc — concrete-subclass instance method dispatch
_sq = _Square(3)
assert _sq.area() == 9; _ledger.append(1)
_sq2 = _Square(5)
assert _sq2.area() == 25; _ledger.append(1)

# 5) contextlib — partial module hasattr surface
#    (ExitStack / closing / AbstractContextManager DIVERGE —
#    moved to spec fixture)
assert hasattr(contextlib, "contextmanager") == True; _ledger.append(1)
assert hasattr(contextlib, "suppress") == True; _ledger.append(1)
assert hasattr(contextlib, "nullcontext") == True; _ledger.append(1)

# 6) contextlib.nullcontext — value-passthrough contract
with contextlib.nullcontext("hello") as _x:
    assert _x == "hello"; _ledger.append(1)
with contextlib.nullcontext(42) as _y:
    assert _y == 42; _ledger.append(1)

# 7) inspect — partial module hasattr surface
#    (Signature / Parameter / ismodule / getdoc / getsource
#    / getfullargspec DIVERGE — moved to spec fixture)
assert hasattr(inspect, "signature") == True; _ledger.append(1)
assert hasattr(inspect, "isfunction") == True; _ledger.append(1)
assert hasattr(inspect, "isclass") == True; _ledger.append(1)
assert hasattr(inspect, "ismethod") == True; _ledger.append(1)
assert hasattr(inspect, "getmembers") == True; _ledger.append(1)

# NB: typing.List[int] / Dict / Set / Tuple / Optional / Union
# / TypeVar / Callable all return None on mamba, typing.Any
# returns a lambda not Any sentinel, hasattr(typing,
# "Iterable") / "Sequence" / "Mapping" False on mamba,
# hasattr(dataclasses, "is_dataclass") / "replace" /
# "make_dataclass" / "MISSING" False, @dataclass auto-init
# returns None for fields on mamba, hasattr(contextlib,
# "ExitStack") / "closing" / "AbstractContextManager" False,
# contextlib.suppress fails to suppress, @contextlib.
# contextmanager yields int 1 not value, hasattr(inspect,
# "Signature") / "Parameter" / "ismodule" / "getdoc" /
# "getsource" / "getfullargspec" False, inspect.signature
# returns empty (), inspect.isfunction returns False on a
# defined function, abc.ABC abstract enforcement is silent
# (Shape() succeeds) — all DIVERGE on mamba — moved to
# the divergence-spec fixture.

print(f"MAMBA_ASSERTION_PASS: test_typing_dataclasses_abc_contextlib_inspect_hasattr_ops {sum(_ledger)} asserts")
