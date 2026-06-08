# Operational AssertionPass seed for the typing-module runtime surface.
# Surface: the basic generic aliases (List, Dict, Tuple, Set, Optional,
# Union, Any) can be imported and are bound to non-None values at
# runtime (they're real objects, not type-checker-only placeholders);
# the Final and ClassVar special forms are also importable; a function
# annotated with `List[int]` and `Optional[int]` runs and returns the
# expected value/None for empty input; Optional[int] is semantically
# equivalent to Union[int, None] (the canonical typing identity);
# annotated local variables `x: List[int] = […]` and `d: Dict[str, int]
# = {…}` round-trip the value cleanly; the modern PEP 604 `int | None`
# syntax co-exists with the older typing.Optional form. Companion to
# lang_typing_aliases (which covers the parameterizable form).
import typing
from typing import List, Dict, Tuple, Set, Optional, Union, Any
_ledger: list[int] = []

# Basic generic aliases exist (importable, non-None)
assert List is not None; _ledger.append(1)
assert Dict is not None; _ledger.append(1)
assert Tuple is not None; _ledger.append(1)
assert Set is not None; _ledger.append(1)
assert Optional is not None; _ledger.append(1)
assert Union is not None; _ledger.append(1)
assert Any is not None; _ledger.append(1)

# Annotation usage — runtime call paths through annotated signatures
def first_int(x: List[int]) -> Optional[int]:
    if not x:
        return None
    return x[0]
assert first_int([1, 2, 3]) == 1; _ledger.append(1)
assert first_int([42]) == 42; _ledger.append(1)
assert first_int([]) == None; _ledger.append(1)

# Optional[int] is semantically equivalent to Union[int, None]
assert Optional[int] == Union[int, None]; _ledger.append(1)
assert Optional[str] == Union[str, None]; _ledger.append(1)

# Final / ClassVar — importable, non-None
from typing import Final, ClassVar
assert Final is not None; _ledger.append(1)
assert ClassVar is not None; _ledger.append(1)

# Annotated local variable — value round-trips
xa: List[int] = [1, 2, 3]
assert xa == [1, 2, 3]; _ledger.append(1)
assert len(xa) == 3; _ledger.append(1)

# Annotated dict
da: Dict[str, int] = {"a": 1, "b": 2}
assert da == {"a": 1, "b": 2}; _ledger.append(1)
assert da["a"] == 1; _ledger.append(1)

# Annotated set
sa: Set[int] = {1, 2, 3}
assert sa == {1, 2, 3}; _ledger.append(1)
assert len(sa) == 3; _ledger.append(1)

# Tuple annotation with fixed-length pair
ta: Tuple[int, str] = (42, "hi")
assert ta == (42, "hi"); _ledger.append(1)

# A function with multiple typing annotations runs and returns the
# expected concrete value
def label_first(values: List[str], default: Optional[str]) -> str:
    if values:
        return values[0]
    if default is None:
        return "EMPTY"
    return default
assert label_first(["a", "b"], None) == "a"; _ledger.append(1)
assert label_first([], None) == "EMPTY"; _ledger.append(1)
assert label_first([], "fallback") == "fallback"; _ledger.append(1)

# PEP 604 — int | None coexists with typing.Optional[int]
def first_int604(x: list[int]) -> int | None:
    if not x:
        return None
    return x[0]
assert first_int604([5, 6]) == 5; _ledger.append(1)
assert first_int604([]) == None; _ledger.append(1)

# typing module itself is a module (has the expected name)
assert typing.__name__ == "typing"; _ledger.append(1)

# Any accepts any value when used as a tag
ax: Any = 42
assert ax == 42; _ledger.append(1)
ax = "hello"
assert ax == "hello"; _ledger.append(1)
ax = [1, 2, 3]
assert ax == [1, 2, 3]; _ledger.append(1)
print(f"MAMBA_ASSERTION_PASS: test_typing_aliases_runtime_ops {sum(_ledger)} asserts")
