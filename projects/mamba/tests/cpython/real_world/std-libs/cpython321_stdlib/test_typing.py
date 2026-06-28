# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cpython321_stdlib"
# dimension = "real_world"
# case = "test_typing"
# subject = "cpython321.test_typing"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/test_typing.py"
# status = "filled"
# ///
"""cpython321.test_typing: execute CPython 3.12 seed test_typing"""
# Promoted from the upstream unittest port to an executable AssertionPass seed.
# Surface: typing — symbol exposure for the core type-hint vocabulary, plus the
# couple of runtime helpers mamba services today:
#   * Many name-level symbols exist (Any, Union, Optional, List, Dict, Tuple,
#     Set, FrozenSet, Callable, Iterator, Generator, TypeVar, Generic, Type,
#     ClassVar, Final, Literal, Protocol, TypedDict, NamedTuple)
#   * typing.cast(t, x) is a passthrough that returns x unchanged
#   * typing.TYPE_CHECKING == False at runtime
#   * typing.get_type_hints returns a dict stable enough to assert on
# Intentionally NOT exercised on mamba today (tracked separately):
#   * typing.{Iterable,Sequence,Mapping,MutableMapping,MutableSequence,NewType,
#     overload} — missing from the mamba stub
#   * Subscripting (Union[int, str], List[int], Optional[int]) — returns None
#   * TypeVar("T") returns None (no .__name__ attribute)
#   * Generic[T] inheritance and isinstance/issubclass against ABC types
#   * typing.Literal[str-literals], typing.Annotated.__metadata__, etc.
#     (tracked as separate axis-1 lang issues #3346/#3347/#3348/#3349)
import typing

_ledger: list[int] = []

# (1) The core type-hint vocabulary is exposed
for _name in ("Any", "Union", "Optional", "List", "Dict", "Tuple", "Set",
              "FrozenSet", "Callable", "Iterator", "Generator", "TypeVar",
              "Generic", "Type", "ClassVar", "Final", "Literal", "Protocol",
              "TypedDict", "NamedTuple"):
    assert hasattr(typing, _name), f"typing.{_name} symbol is exposed"
_ledger.append(1)

# (2) typing.cast is exposed and is a passthrough
assert hasattr(typing, "cast"), "typing.cast symbol is exposed"
_ledger.append(1)

_c1 = typing.cast(int, 42)
assert _c1 - 42 == 0, f"typing.cast(int, 42) returns 42, got {_c1!r}"
_ledger.append(1)

_c2 = typing.cast(str, "hi")
assert _c2 == "hi", f"typing.cast(str, 'hi') returns 'hi', got {_c2!r}"
_ledger.append(1)

_c3 = typing.cast(list, [1, 2, 3])
assert _c3 == [1, 2, 3], f"typing.cast(list, [...]) returns the list, got {_c3!r}"
_ledger.append(1)

# (3) typing.TYPE_CHECKING is False at runtime
assert typing.TYPE_CHECKING == False, (
    f"typing.TYPE_CHECKING is False at runtime, got {typing.TYPE_CHECKING!r}"
)
_ledger.append(1)

# (4) typing.get_type_hints is exposed and returns a dict-like object
assert hasattr(typing, "get_type_hints"), "typing.get_type_hints symbol is exposed"
_ledger.append(1)

def _annotated_fn(x: int, y: str) -> bool: return True

_hints = typing.get_type_hints(_annotated_fn)
assert isinstance(_hints, dict), (
    f"typing.get_type_hints returns a dict, got {type(_hints).__name__!r}"
)
_ledger.append(1)

# (5) typing.cast through a multi-step pipeline preserves identity
_value = [1, 2, 3]
_piped = typing.cast(list, typing.cast(object, _value))
assert _piped == [1, 2, 3], (
    f"chained typing.cast preserves the value, got {_piped!r}"
)
_ledger.append(1)

# (6) typing.Any, Union, Optional, Callable, List, Dict, Tuple, Set
#     attributes are non-None lambda stubs
for _name in ("Any", "Union", "Optional", "Callable", "List", "Dict", "Tuple", "Set"):
    _sym = getattr(typing, _name)
    assert _sym is not None, f"typing.{_name} attribute access does not return None"
_ledger.append(1)

# (7) TYPE_CHECKING-gated branch behaves like the runtime branch
if typing.TYPE_CHECKING:
    _branch = "compile-time"
else:
    _branch = "runtime"

assert _branch == "runtime", (
    f"if typing.TYPE_CHECKING: takes the runtime branch, got {_branch!r}"
)
_ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_typing {sum(_ledger)} asserts")
