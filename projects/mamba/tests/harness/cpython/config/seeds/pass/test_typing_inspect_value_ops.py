# Atomic 291 pass conformance — typing module (hasattr List/Dict/
# Set/Tuple/Optional/Union/Callable/Any/TypeVar/Generic/Iterator/
# ClassVar/Final/Literal/Protocol/TypedDict/NamedTuple/cast/
# get_type_hints + cast returns value) + inspect module (hasattr
# signature/isfunction/ismethod/isclass/getmembers).
# All asserts match between CPython 3.12 and mamba.
import typing
import inspect


_ledger: list[int] = []

# 1) typing — hasattr generic alias surface
assert hasattr(typing, "List") == True; _ledger.append(1)
assert hasattr(typing, "Dict") == True; _ledger.append(1)
assert hasattr(typing, "Set") == True; _ledger.append(1)
assert hasattr(typing, "Tuple") == True; _ledger.append(1)
assert hasattr(typing, "Optional") == True; _ledger.append(1)
assert hasattr(typing, "Union") == True; _ledger.append(1)
assert hasattr(typing, "Callable") == True; _ledger.append(1)
assert hasattr(typing, "Any") == True; _ledger.append(1)

# 2) typing — hasattr generic/type surface
assert hasattr(typing, "TypeVar") == True; _ledger.append(1)
assert hasattr(typing, "Generic") == True; _ledger.append(1)
assert hasattr(typing, "Iterator") == True; _ledger.append(1)
assert hasattr(typing, "ClassVar") == True; _ledger.append(1)
assert hasattr(typing, "Final") == True; _ledger.append(1)
assert hasattr(typing, "Literal") == True; _ledger.append(1)

# 3) typing — hasattr special-form/class surface
assert hasattr(typing, "Protocol") == True; _ledger.append(1)
assert hasattr(typing, "TypedDict") == True; _ledger.append(1)
assert hasattr(typing, "NamedTuple") == True; _ledger.append(1)

# 4) typing — hasattr helper surface
assert hasattr(typing, "cast") == True; _ledger.append(1)
assert hasattr(typing, "get_type_hints") == True; _ledger.append(1)

# 5) typing — value contracts
assert typing.cast(int, 5) == 5; _ledger.append(1)

# 6) inspect — hasattr predicate/signature surface
assert hasattr(inspect, "signature") == True; _ledger.append(1)
assert hasattr(inspect, "isfunction") == True; _ledger.append(1)
assert hasattr(inspect, "ismethod") == True; _ledger.append(1)
assert hasattr(inspect, "isclass") == True; _ledger.append(1)
assert hasattr(inspect, "getmembers") == True; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_typing_inspect_value_ops {sum(_ledger)} asserts")
