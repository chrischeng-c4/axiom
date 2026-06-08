# Operational AssertionPass seed for typing module aliases used as
# annotation surfaces. Surface: List/Dict/Tuple/Set/Any annotations
# round-trip values; no narrowing or generic enforcement required.
from typing import List, Dict, Tuple, Set, Any
_ledger: list[int] = []

xs: List[int] = [1, 2, 3]
assert xs == [1, 2, 3]; _ledger.append(1)

d: Dict[str, int] = {"a": 1, "b": 2}
assert d["a"] == 1; _ledger.append(1)

t: Tuple[int, str] = (1, "x")
assert t == (1, "x"); _ledger.append(1)

s: Set[int] = {1, 2, 3}
assert 2 in s; _ledger.append(1)
assert len(s) == 3; _ledger.append(1)

a: Any = 1
assert a == 1; _ledger.append(1)
a = "hello"
assert a == "hello"; _ledger.append(1)
a = [9, 8, 7]
assert a == [9, 8, 7]; _ledger.append(1)

# Lowercase builtin-generic annotations (3.9+)
ys: list[str] = ["a", "b"]
assert len(ys) == 2; _ledger.append(1)
m: dict[int, int] = {1: 10}
assert m[1] == 10; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_typing_aliases {sum(_ledger)} asserts")
