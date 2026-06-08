"""Behavior contract for language type annotations (PEP 484/526/563).

Each block tests one semantic rule. Expected values are what CPython
3.12 produces. mamba PASS = matches CPython exactly.

# type-regime: monomorphic
"""

import typing
from typing import Optional, Union, List, Dict, Callable

# Rule 1: Annotations don't affect runtime behavior
def _fn(x: int, y: str = "hi") -> bool:
    return True

assert _fn(1) == True, f"annotated fn = {_fn(1)!r}"
assert _fn(1, "bye") == True, f"annotated fn2 = {_fn(1,'bye')!r}"
# Wrong types still work (no enforcement)
assert _fn("not_int") == True, f"wrong type still works"  # type: ignore[arg-type]

# Rule 2: __annotations__ dict reflects declared annotations
def _typed(a: int, b: float, c: str) -> bool:
    return True

_ann = _typed.__annotations__
assert "a" in _ann, "a in annotations"
assert "b" in _ann, "b in annotations"
assert "c" in _ann, "c in annotations"
assert "return" in _ann, "return in annotations"
assert _ann["return"] is bool, f"return annotation = {_ann['return']!r}"

# Rule 3: Variable annotations — PEP 526
class _Container:
    value: int = 0
    name: str = "default"

_c = _Container()
assert _c.value == 0, f"value = {_c.value!r}"
assert _c.name == "default", f"name = {_c.name!r}"
assert "value" in _Container.__annotations__, "value in class annotations"
assert "name" in _Container.__annotations__, "name in class annotations"

# Rule 4: get_type_hints returns resolved hints
def _hints_fn(x: int, y: Optional[str] = None) -> List[int]:
    return []

_hints = typing.get_type_hints(_hints_fn)
assert isinstance(_hints, dict), f"hints type = {type(_hints)!r}"
assert "x" in _hints, "x in hints"
assert "y" in _hints, "y in hints"
assert "return" in _hints, "return in hints"

# Rule 5: Optional[X] is Union[X, None]
assert Optional[int] == Union[int, None], "Optional[int] == Union[int, None]"

# Rule 6: Callable annotation
def _apply(fn: Callable[[int], int], v: int) -> int:
    return fn(v)

assert _apply(lambda x: x * 2, 5) == 10, f"apply = {_apply(lambda x: x*2, 5)!r}"

# Rule 7: Annotations are not evaluated at runtime for classes by default
class _Lazy:
    # Forward reference: int is available so it resolves fine
    def method(self) -> int:
        return 42

assert _Lazy().method() == 42, f"lazy method = {_Lazy().method()!r}"
assert "return" in _Lazy.method.__annotations__, "return in method annotations"

# Rule 8: Dict annotation generic
def _word_count(text: str) -> Dict[str, int]:
    result: Dict[str, int] = {}
    for word in text.split():
        result[word] = result.get(word, 0) + 1
    return result

_wc = _word_count("a b a c a b")
assert _wc == {"a": 3, "b": 2, "c": 1}, f"word_count = {_wc!r}"

print("behavior OK")
