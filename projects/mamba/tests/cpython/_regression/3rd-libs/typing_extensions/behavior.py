"""Behavior contract for third-party typing_extensions package.

Each block tests one semantic rule. Expected values are what CPython
3.12 produces. mamba PASS = matches CPython exactly.

# type-regime: monomorphic
"""

import typing_extensions as te  # type: ignore[import]

# Rule 1: Protocol with runtime_checkable supports isinstance
@te.runtime_checkable
class _Sized(te.Protocol):
    def __len__(self) -> int: ...

assert isinstance([], _Sized), "list is Sized"
assert isinstance({}, _Sized), "dict is Sized"
assert not isinstance(42, _Sized), "int is not Sized"

# Rule 2: TypedDict creates a dict subclass schema
class _Config(te.TypedDict):
    host: str
    port: int

_c: _Config = {"host": "localhost", "port": 8080}
assert _c["host"] == "localhost", "TypedDict host"
assert _c["port"] == 8080, "TypedDict port"

# Rule 3: override decorator passes through the method
class _Animal:
    def speak(self) -> str:
        return "..."

class _Dog(_Animal):
    @te.override
    def speak(self) -> str:
        return "woof"

assert _Dog().speak() == "woof", "override method"

# Rule 4: Literal restricts values at type-check time; runtime is normal
def _f4(x: te.Literal["a", "b", "c"]) -> str:
    return x.upper()

assert _f4("a") == "A", "Literal param a"
assert _f4("b") == "B", "Literal param b"

# Rule 5: Annotated wraps a type with metadata
def _make_annotated():
    _T = te.Annotated[int, "positive"]
    return _T

_ann = _make_annotated()
assert hasattr(_ann, "__metadata__"), "Annotated has __metadata__"
assert _ann.__metadata__ == ("positive",), f"metadata = {_ann.__metadata__!r}"

# Rule 6: get_overloads returns overloads registered for a function
from typing_extensions import overload  # type: ignore[import]

@overload
def _process(x: int) -> int: ...
@overload
def _process(x: str) -> str: ...
def _process(x):
    return x

_overloads = te.get_overloads(_process)
assert isinstance(_overloads, list), f"get_overloads type = {type(_overloads)!r}"
assert len(_overloads) == 2, f"two overloads: {len(_overloads)!r}"

# Rule 7: assert_type is a no-op at runtime (returns its argument)
_result7 = te.assert_type(42, int)
assert _result7 == 42, f"assert_type returns value: {_result7!r}"

# Rule 8: Module attributes are identity-stable across repeated reads
_attrs = ["Protocol", "TypedDict", "runtime_checkable", "override"]
_refs = {_a: getattr(te, _a) for _a in _attrs}
for _ in range(10):
    for _a in _attrs:
        assert getattr(te, _a) is _refs[_a], f"{_a} stable identity"

print("behavior OK")
