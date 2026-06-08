"""Surface contract for language type annotations (PEP 484/526/563).

# type-regime: monomorphic

Probes: function parameter annotations, return annotation, variable annotation,
__annotations__ dict, get_type_hints, typing module basics.
CPython 3.12 is the oracle.
"""

from typing import Optional, Union, List, Dict, Tuple, Any
import typing

# Function parameter and return annotations
def _add(a: int, b: int) -> int:
    return a + b

assert callable(_add), "_add not callable"
assert _add(2, 3) == 5, f"_add = {_add(2,3)!r}"

# __annotations__ dict on function
assert hasattr(_add, "__annotations__"), "_add missing __annotations__"
_ann = _add.__annotations__
assert isinstance(_ann, dict), f"__annotations__ type = {type(_ann)!r}"
assert "return" in _ann, "__annotations__ missing 'return'"

# Variable annotations (PEP 526) — stored in module __annotations__
_count: int = 0
_name: str = "hello"
assert _count == 0, f"_count = {_count!r}"
assert _name == "hello", f"_name = {_name!r}"

# Optional is accessible
def _greet(name: Optional[str] = None) -> str:
    return f"hi {name or 'world'}"

assert _greet() == "hi world", f"greet() = {_greet()!r}"
assert _greet("Alice") == "hi Alice", f"greet(Alice) = {_greet('Alice')!r}"

# Union type annotation
def _describe(v: Union[int, str]) -> str:
    return str(v)

assert _describe(42) == "42", f"describe(42) = {_describe(42)!r}"
assert _describe("hi") == "hi", f"describe(hi) = {_describe('hi')!r}"

# Annotations don't enforce types at runtime
def _typed(x: int) -> int:
    return x

_result = _typed("string")  # type: ignore[arg-type]  — CPython allows this
assert _result == "string", "annotations are NOT enforced at runtime"

# typing.get_type_hints
_hints = typing.get_type_hints(_add)
assert isinstance(_hints, dict), f"get_type_hints type = {type(_hints)!r}"

# List, Dict, Tuple from typing work as generic aliases
def _sum_list(items: List[int]) -> int:
    return sum(items)

assert _sum_list([1, 2, 3]) == 6, f"sum_list = {_sum_list([1,2,3])!r}"

print("surface OK")
