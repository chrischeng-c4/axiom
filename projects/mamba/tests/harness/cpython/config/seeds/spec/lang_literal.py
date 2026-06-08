# lang_literal.py — #3348 axis-1 lang Literal + Final + Annotated +
# NewType AssertionPass seed.
#
# Mamba-authored seed exercising the typing-level surface called out in
# the issue:
#   * Literal[str-literals]
#   * Final
#   * Annotated.__metadata__
#   * NewType
#
# Contract placement: `spec/` — pins outcome Fail. Mamba runtime gap
# #3495 (Literal[str-literals], Annotated.__metadata__, NewType all
# unimplemented) blocks AssertionPass today. Once #3495 lands and the
# seed flips to AssertionPass on mamba, drift detection prompts a
# `git mv spec/lang_literal.py pass/lang_literal.py`.
#
# Surface coverage (asserts run at module scope; no helper closures per
# the mamba top-level def() quirk in test_math.py):
#   1. Module identity + import.
#   2. Literal[int / str / bool] — typing.get_args returns the literal
#      values; runtime arg order preserved.
#   3. Final — wraps a type; typing.get_origin returns Final.
#   4. Annotated — preserves __metadata__ tuple in declaration order.
#   5. Annotated — get_type_hints(..., include_extras=True) preserves
#      metadata; include_extras=False strips it.
#   6. NewType — call form returns the value unchanged; .__name__ and
#      .__supertype__ exposed.
#
# Contract with `cpython_lib_test_runner.rs`:
#   - Each `assert` runs at top level. Inverting any assert raises
#     AssertionError → non-zero exit → runner classifies as `Fail`.
#   - After every assertion has executed, the seed emits
#     `MAMBA_ASSERTION_PASS: lang_literal N asserts` to stdout.

import typing
from typing import Annotated, Final, Literal, NewType, get_args, get_origin

_ledger: list[int] = []

# 1. Module identity.
assert typing.__name__ == "typing", "typing.__name__"
_ledger.append(1)

# 2. Literal[int].
_LitInt = Literal[1, 2, 3]
_args = get_args(_LitInt)
assert _args == (1, 2, 3), "Literal[1,2,3] preserves arg order"
_ledger.append(1)
assert get_origin(_LitInt) is Literal, "Literal[...] origin is Literal"
_ledger.append(1)

# 2b. Literal[str] — string literals.
_LitStr = Literal["red", "green", "blue"]
_args_s = get_args(_LitStr)
assert _args_s == ("red", "green", "blue"), (
    "Literal[str,str,str] preserves arg order"
)
_ledger.append(1)
assert get_origin(_LitStr) is Literal, "Literal[str-literals] origin is Literal"
_ledger.append(1)

# 2c. Literal[bool] — True / False.
_LitBool = Literal[True, False]
_args_b = get_args(_LitBool)
assert _args_b == (True, False), "Literal[True,False] preserves arg order"
_ledger.append(1)


# 3. Final[T] — typing.get_origin returns Final.
_FinalInt = Final[int]
assert get_origin(_FinalInt) is Final, "Final[int] origin is Final"
_ledger.append(1)
_final_args = get_args(_FinalInt)
assert _final_args == (int,), "Final[int] args == (int,)"
_ledger.append(1)


# 4. Annotated — preserves underlying type as get_args()[0]; metadata
# tuple via __metadata__.
_A1 = Annotated[int, "positive", 42]
# get_args returns (underlying_type, *metadata).
_a1_args = get_args(_A1)
assert _a1_args == (int, "positive", 42), (
    "Annotated.get_args returns (type, *metadata) in declaration order"
)
_ledger.append(1)
assert _a1_args[0] is int, "Annotated.get_args()[0] is the underlying type"
_ledger.append(1)
# Annotated.__metadata__ holds just the metadata tuple.
assert hasattr(_A1, "__metadata__"), "Annotated exposes __metadata__"
_ledger.append(1)
assert _A1.__metadata__ == ("positive", 42), (  # type: ignore[attr-defined]
    "Annotated.__metadata__ is the metadata tuple in declaration order"
)
_ledger.append(1)

# 4b. Annotated with object metadata.
class _Marker:
    def __init__(self, tag: str) -> None:
        self.tag = tag


_m_obj = _Marker("flag")
_A2 = Annotated[str, _m_obj]
assert _A2.__metadata__ == (_m_obj,), (  # type: ignore[attr-defined]
    "Annotated.__metadata__ preserves object identity"
)
_ledger.append(1)
assert _A2.__metadata__[0].tag == "flag", (  # type: ignore[attr-defined]
    "metadata object attrs accessible from Annotated"
)
_ledger.append(1)


# 5. get_type_hints — include_extras=True preserves Annotated metadata.
def _annotated_fn(x: Annotated[int, "positive"]) -> int:
    return x + 1


_hints_keep = typing.get_type_hints(_annotated_fn, include_extras=True)
# x annotation under include_extras is the full Annotated alias.
assert "x" in _hints_keep, "get_type_hints exposes 'x' hint"
_ledger.append(1)
_x_hint = _hints_keep["x"]
assert hasattr(_x_hint, "__metadata__"), (
    "include_extras=True keeps __metadata__ on the hint"
)
_ledger.append(1)
assert _x_hint.__metadata__ == ("positive",), (
    "include_extras=True preserves Annotated metadata"
)
_ledger.append(1)
# include_extras=False strips metadata back to the bare type.
_hints_strip = typing.get_type_hints(_annotated_fn, include_extras=False)
assert _hints_strip["x"] is int, (
    "include_extras=False strips Annotated back to underlying type"
)
_ledger.append(1)


# 6. NewType — call form acts as identity; .__name__ and .__supertype__ set.
UserId = NewType("UserId", int)
assert UserId.__name__ == "UserId", "NewType.__name__ matches call argument"
_ledger.append(1)
assert UserId.__supertype__ is int, (  # type: ignore[attr-defined]
    "NewType.__supertype__ is the underlying type"
)
_ledger.append(1)
# Call form is identity (no boxing).
_uid = UserId(7)
assert _uid - 7 == 0, "NewType(value) returns the value unchanged (boxed-dodge)"
_ledger.append(1)
# str-based NewType.
ScreenName = NewType("ScreenName", str)
_sn = ScreenName("alice")
assert _sn == "alice", "NewType(str) returns the value unchanged"
_ledger.append(1)
assert ScreenName.__supertype__ is str, (  # type: ignore[attr-defined]
    "ScreenName.__supertype__ is str"
)
_ledger.append(1)

# Emit the proof-of-execution marker. Per `ASSERTION_PASS_MARKERS` in
# `cpython_lib_test_runner.rs`, presence of this token escalates the
# outcome from ImportPass to AssertionPass.
print(f"MAMBA_ASSERTION_PASS: lang_literal {len(_ledger)} asserts")
