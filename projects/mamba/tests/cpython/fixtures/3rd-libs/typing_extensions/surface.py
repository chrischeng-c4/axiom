"""Surface contract for third-party typing_extensions package.

# type-regime: monomorphic

Probes: Protocol, TypedDict, runtime_checkable, override, TypeAlias,
Annotated, Literal, TypeVar, ParamSpec, TypeVarTuple, Unpack,
LiteralString, Never, assert_never, assert_type, dataclass_transform,
get_overloads, reveal_type.
CPython 3.12 is the oracle.
"""

import typing_extensions as te

# Core type-system constructs
assert hasattr(te, "Protocol"), "Protocol"
assert hasattr(te, "TypedDict"), "TypedDict"
assert hasattr(te, "runtime_checkable"), "runtime_checkable"
assert hasattr(te, "override"), "override"
assert hasattr(te, "TypeAlias"), "TypeAlias"
assert hasattr(te, "Annotated"), "Annotated"
assert hasattr(te, "Literal"), "Literal"
assert hasattr(te, "TypeVar"), "TypeVar"
assert hasattr(te, "ParamSpec"), "ParamSpec"
assert hasattr(te, "TypeVarTuple"), "TypeVarTuple"
assert hasattr(te, "Unpack"), "Unpack"
assert hasattr(te, "LiteralString"), "LiteralString"
assert hasattr(te, "Never"), "Never"
assert hasattr(te, "assert_never"), "assert_never"
assert hasattr(te, "assert_type"), "assert_type"
assert hasattr(te, "dataclass_transform"), "dataclass_transform"
assert hasattr(te, "get_overloads"), "get_overloads"
assert hasattr(te, "reveal_type"), "reveal_type"
assert hasattr(te, "overload"), "overload"
assert hasattr(te, "final"), "final"
assert hasattr(te, "Final"), "Final"
assert hasattr(te, "ClassVar"), "ClassVar"
assert hasattr(te, "Self"), "Self"
assert hasattr(te, "Concatenate"), "Concatenate"

# Protocol can be subclassed
@te.runtime_checkable
class _Drawable(te.Protocol):
    def draw(self) -> None: ...

class _Circle:
    def draw(self) -> None:
        pass

_c = _Circle()
assert isinstance(_c, _Drawable), "Circle is Drawable via Protocol"

# TypedDict
class _Point(te.TypedDict):
    x: int
    y: int

_p: _Point = {"x": 1, "y": 2}
assert _p["x"] == 1 and _p["y"] == 2, "TypedDict usage"

# override decorator
class _Base:
    def method(self) -> int:
        return 0

class _Derived(_Base):
    @te.override
    def method(self) -> int:
        return 1

assert _Derived().method() == 1, "override method works"

# Module attributes are identity-stable (benched in hot loop)
_proto_ref = te.Protocol
_td_ref = te.TypedDict
_rc_ref = te.runtime_checkable
_ov_ref = te.override
assert te.Protocol is _proto_ref, "Protocol stable"
assert te.TypedDict is _td_ref, "TypedDict stable"
assert te.runtime_checkable is _rc_ref, "runtime_checkable stable"
assert te.override is _ov_ref, "override stable"

print("surface OK")
