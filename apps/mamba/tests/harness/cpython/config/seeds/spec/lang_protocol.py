# lang_protocol.py — #3347 axis-1 lang Protocol + runtime_checkable seed.
#
# Mamba-authored seed exercising the structural-typing surface called
# out in the issue:
#   class P(Protocol): def f(self) -> int: ...
#   @runtime_checkable enables isinstance(x, P)
#   Structural match (no nominal inheritance needed)
#   Protocol with attribute (not method)
#
# Contract placement: `spec/` — pins outcome Fail. Mamba runtime gap
# #3494 (@runtime_checkable Protocol isinstance always returns False)
# blocks AssertionPass today. Once #3494 lands and this seed flips to
# AssertionPass on mamba, drift detection prompts a
# `git mv spec/lang_protocol.py pass/lang_protocol.py`.
#
# Surface coverage (asserts run at module scope; no helper closures per
# the mamba top-level def() quirk in test_math.py):
#   1. Module identity + import.
#   2. Plain Protocol (no @runtime_checkable) — structural definition
#      compiles; isinstance against it raises TypeError per PEP 544.
#   3. @runtime_checkable + isinstance — duck-typed match without
#      nominal inheritance.
#   4. @runtime_checkable + isinstance — negative case (missing method).
#   5. Protocol with attribute (not method).
#   6. Multi-method Protocol — all methods must be present.
#   7. Protocol __subclasshook__ via issubclass on a class with the
#      structural shape.
#
# Contract with `cpython_lib_test_runner.rs`:
#   - Each `assert` runs at top level. Inverting any assert raises
#     AssertionError → non-zero exit → runner classifies as `Fail`.
#   - After every assertion has executed, the seed emits
#     `MAMBA_ASSERTION_PASS: lang_protocol N asserts` to stdout.

from typing import Protocol, runtime_checkable

_ledger: list[int] = []


# 2. Plain Protocol — no @runtime_checkable.
class Lengthy(Protocol):
    def __len__(self) -> int: ...


# Plain Protocol cannot be used with isinstance — should raise TypeError.
_raised_plain = False
try:
    isinstance("hello", Lengthy)  # type: ignore[arg-type]
except TypeError:
    _raised_plain = True
assert _raised_plain == True, (
    "plain Protocol (no @runtime_checkable) rejects isinstance with TypeError"
)
_ledger.append(1)


# 3. @runtime_checkable + isinstance — positive structural match.
@runtime_checkable
class Sized(Protocol):
    def __len__(self) -> int: ...


# Built-in list has __len__ — structural match without nominal inheritance.
assert isinstance([1, 2, 3], Sized) == True, (
    "@runtime_checkable Protocol matches list via __len__"
)
_ledger.append(1)
assert isinstance("hello", Sized) == True, (
    "@runtime_checkable Protocol matches str via __len__"
)
_ledger.append(1)
assert isinstance({}, Sized) == True, (
    "@runtime_checkable Protocol matches dict via __len__"
)
_ledger.append(1)
assert isinstance((1, 2), Sized) == True, (
    "@runtime_checkable Protocol matches tuple via __len__"
)
_ledger.append(1)


# 4. Negative case — int lacks __len__.
assert isinstance(42, Sized) == False, (
    "@runtime_checkable Protocol rejects int (no __len__)"
)
_ledger.append(1)
# Also a user class with no __len__.
class _NoLen:
    pass


_nl = _NoLen()
assert isinstance(_nl, Sized) == False, (
    "@runtime_checkable Protocol rejects user class lacking __len__"
)
_ledger.append(1)


# 5. Protocol with a method that returns int — structural duck typing.
@runtime_checkable
class Computer(Protocol):
    def compute(self) -> int: ...


class _Adder:
    def compute(self) -> int:
        return 7


_ad = _Adder()
assert isinstance(_ad, Computer) == True, (
    "Computer matches _Adder via structural shape (no nominal Computer base)"
)
_ledger.append(1)
# Calling the matched method still works as normal Python attribute lookup.
assert _ad.compute() - 7 == 0, "_Adder.compute() returns 7 (boxed-dodge)"
_ledger.append(1)


# 6. Attribute-bearing Protocol — runtime_checkable supports attrs.
@runtime_checkable
class HasName(Protocol):
    name: str


class _Named:
    name: str = "alice"


_n = _Named()
assert isinstance(_n, HasName) == True, (
    "@runtime_checkable Protocol matches via instance attr 'name'"
)
_ledger.append(1)
# Class without `name` attr fails the match.
class _Anon:
    pass


_an = _Anon()
assert isinstance(_an, HasName) == False, (
    "@runtime_checkable Protocol rejects class lacking 'name' attr"
)
_ledger.append(1)


# 7. Multi-method Protocol — all methods must be present.
@runtime_checkable
class ReadCloser(Protocol):
    def read(self) -> bytes: ...
    def close(self) -> None: ...


class _FullReader:
    def read(self) -> bytes:
        return b""
    def close(self) -> None:
        pass


assert isinstance(_FullReader(), ReadCloser) == True, (
    "multi-method Protocol matches when all methods present"
)
_ledger.append(1)


class _PartialReader:
    def read(self) -> bytes:
        return b""
    # NO close — should fail the structural check.


assert isinstance(_PartialReader(), ReadCloser) == False, (
    "multi-method Protocol rejects when a method is missing"
)
_ledger.append(1)


# 8. issubclass support on @runtime_checkable Protocol — Sized matches list.
assert issubclass(list, Sized) == True, (
    "@runtime_checkable Protocol issubclass: list has __len__"
)
_ledger.append(1)
assert issubclass(int, Sized) == False, (
    "@runtime_checkable Protocol issubclass: int lacks __len__"
)
_ledger.append(1)

# Emit the proof-of-execution marker. Per `ASSERTION_PASS_MARKERS` in
# `cpython_lib_test_runner.rs`, presence of this token escalates the
# outcome from ImportPass to AssertionPass.
print(f"MAMBA_ASSERTION_PASS: lang_protocol {len(_ledger)} asserts")
