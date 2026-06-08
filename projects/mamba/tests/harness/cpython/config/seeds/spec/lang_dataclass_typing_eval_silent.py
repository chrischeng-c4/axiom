# Operational AssertionPass seed for SILENT divergences across the
# modeling / pattern-matching / introspection sextet pinned by
# atomic 159: `dataclasses` (the documented `replace` / `Field` /
# `FrozenInstanceError` / `is_dataclass` / `MISSING` helper +
# class identity), `typing` (the documented `Any` repr +
# `TypedDict` / `NamedTuple` instance construction), the
# printf-style `%e` scientific-notation specifier, the
# `callable` built-in (documented to return False on bare
# integers), and the `eval` / `exec` / `compile` built-ins
# (the documented globals-binding / namespace-population /
# compiled-code contracts).
#
# The matching subset (dataclasses.dataclass / asdict /
# astuple / fields / field hasattr surface, typing
# TYPE_CHECKING + TypedDict / NamedTuple / Protocol / Generic /
# TypeVar / cast / Optional / Union / List / Dict / Tuple /
# Callable / Any hasattr surface, match / case literal-guard-
# list-dict-wildcard patterns, printf-style `%d` / `%s` / `%.2f`
# / multi-arg tuple / padded / zero-padded / `%x` / `%X` / `%o`
# / named-dict / `%r`, hasattr / getattr / setattr / delattr /
# vars / dir / globals / locals, callable lambda / callable
# str / callable list, eval simple arithmetic) is covered by
# `test_dataclass_typing_match_dir_value_ops`; this fixture
# pins the CPython-only contracts that mamba currently elides.
#
# Surface (CPython AssertionPass; mamba diverges silently):
#   • hasattr(dataclasses, "replace") is True — documented
#     immutable-update helper (mamba: False);
#   • hasattr(dataclasses, "Field") is True — documented
#     metadata-field descriptor (mamba: False);
#   • hasattr(dataclasses, "FrozenInstanceError") is True —
#     documented exception (mamba: False);
#   • hasattr(dataclasses, "is_dataclass") is True —
#     documented runtime predicate (mamba: False);
#   • hasattr(dataclasses, "MISSING") is True — documented
#     sentinel for missing-default field (mamba: False);
#   • repr(typing.Any) == "typing.Any" — class-identity repr
#     (mamba: returns a lambda-function repr, e.g.
#     "<function <lambda> at 0x...>");
#   • TD(name="alice", age=30)["name"] == "alice" — TypedDict
#     keyword-arg construction populates the underlying dict
#     (mamba: TD instance is empty, td["name"] returns None);
#   • isinstance(TD(name="alice", age=30), dict) is True —
#     TypedDict instance is a dict (mamba: returns False);
#   • NTuple(1, 2).x == 1 — NamedTuple attribute access on a
#     positional construction (mamba: nt.x returns None,
#     positional args dropped on the floor);
#   • NTuple(1, 2)[0] == 1 — NamedTuple positional indexing
#     (mamba: nt[0] returns None);
#   • "%e" % 1e6 == "1.000000e+06" — printf-style scientific-
#     notation specifier (mamba: returns literal string "%e",
#     specifier not applied);
#   • callable(1) is False — int instances are not callable
#     (mamba: returns True);
#   • eval("x*2", {"x": 21}) == 42 — eval with explicit
#     globals dict (mamba: returns None, globals binding
#     dropped);
#   • exec("y = 100", ns) populates ns["y"] == 100 —
#     namespace-population contract (mamba: ns.get("y")
#     returns None, exec doesn't populate the namespace);
#   • compile + eval(code, {"x": 41}) == 42 — compiled-code
#     evaluation contract (mamba: returns None).
import dataclasses as _dataclasses_mod
import typing as _typing_mod
from typing import Any

# Module bindings retyped as `Any` to bypass Pyright stub-driven
# narrowing — every spec contract below probes documented public
# class identifiers / module-level helpers / instance methods
# that mamba's bundled type stubs do not surface accurately.
dataclasses: Any = _dataclasses_mod
typing: Any = _typing_mod


# typing.TypedDict / NamedTuple subclasses must live at module
# level — mamba elides classes declared inside try/except blocks.
class TD(typing.TypedDict):
    name: str
    age: int


class NTuple(typing.NamedTuple):
    x: int
    y: int


_ledger: list[int] = []

# 1) dataclasses — documented helper attribute surface
assert hasattr(dataclasses, "replace") == True; _ledger.append(1)
assert hasattr(dataclasses, "Field") == True; _ledger.append(1)
assert hasattr(dataclasses, "FrozenInstanceError") == True; _ledger.append(1)
assert hasattr(dataclasses, "is_dataclass") == True; _ledger.append(1)
assert hasattr(dataclasses, "MISSING") == True; _ledger.append(1)

# 2) typing.Any — canonical repr
assert repr(typing.Any) == "typing.Any"; _ledger.append(1)

# 3) typing.TypedDict — keyword-arg construction populates dict
_td = TD(name="alice", age=30)
assert _td["name"] == "alice"; _ledger.append(1)
assert _td["age"] == 30; _ledger.append(1)
assert isinstance(_td, dict) == True; _ledger.append(1)

# 4) typing.NamedTuple — positional attribute + indexing
_nt = NTuple(1, 2)
assert _nt.x == 1; _ledger.append(1)
assert _nt.y == 2; _ledger.append(1)
assert _nt[0] == 1; _ledger.append(1)
assert _nt[1] == 2; _ledger.append(1)

# 5) printf-style `%e` — scientific-notation specifier
assert "%e" % 1e6 == "1.000000e+06"; _ledger.append(1)

# 6) callable — int is not callable
assert callable(1) == False; _ledger.append(1)

# 7) eval — explicit globals binding
assert eval("x*2", {"x": 21}) == 42; _ledger.append(1)

# 8) exec — populates supplied namespace
_ns: dict[str, Any] = {}
exec("y = 100", _ns)
assert _ns.get("y") == 100; _ledger.append(1)

# 9) compile + eval(code, globals) — compiled-code evaluation
_code = compile("x+1", "<test>", "eval")
assert eval(_code, {"x": 41}) == 42; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_dataclass_typing_eval_silent {sum(_ledger)} asserts")
