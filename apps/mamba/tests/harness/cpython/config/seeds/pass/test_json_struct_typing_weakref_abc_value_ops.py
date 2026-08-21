# Atomic 236 pass conformance — json / struct / typing / weakref / abc /
# enum / dataclasses / fractions / decimal surface + value ops that match
# between CPython 3.12 and mamba.
import json
import struct
import typing
import weakref
import abc
import enum
import dataclasses
import fractions
import decimal


_ledger: list[int] = []

# 1) json value ops
assert json.dumps([1, 2, 3]) == "[1, 2, 3]"; _ledger.append(1)
assert json.dumps("hello") == '"hello"'; _ledger.append(1)
assert json.dumps(42) == "42"; _ledger.append(1)
assert json.dumps(True) == "true"; _ledger.append(1)
assert json.dumps(False) == "false"; _ledger.append(1)
assert json.dumps(None) == "null"; _ledger.append(1)
assert json.dumps({"a": 1}) == '{"a": 1}'; _ledger.append(1)
assert json.loads("[1, 2, 3]") == [1, 2, 3]; _ledger.append(1)
assert json.loads('{"a": 1, "b": 2}') == {"a": 1, "b": 2}; _ledger.append(1)
assert json.loads('"hello"') == "hello"; _ledger.append(1)
assert json.loads("42") == 42; _ledger.append(1)
assert json.loads("true") == True; _ledger.append(1)
assert json.loads("false") == False; _ledger.append(1)
assert json.loads("null") is None; _ledger.append(1)
assert json.loads('{"a": [1, 2, {"b": "x"}]}') == {"a": [1, 2, {"b": "x"}]}; _ledger.append(1)
assert json.dumps({"b": 1, "a": 2}, sort_keys=True) == '{"a": 2, "b": 1}'; _ledger.append(1)
assert hasattr(json, "dumps") == True; _ledger.append(1)
assert hasattr(json, "loads") == True; _ledger.append(1)
assert hasattr(json, "dump") == True; _ledger.append(1)
assert hasattr(json, "load") == True; _ledger.append(1)
assert hasattr(json, "JSONDecodeError") == True; _ledger.append(1)
assert hasattr(json, "JSONEncoder") == True; _ledger.append(1)
assert hasattr(json, "JSONDecoder") == True; _ledger.append(1)

# 2) struct value ops + surface
assert struct.pack(">i", 42) == b"\x00\x00\x00\x2a"; _ledger.append(1)
assert struct.unpack(">i", b"\x00\x00\x00\x2a") == (42,); _ledger.append(1)
assert struct.calcsize(">i") == 4; _ledger.append(1)
assert struct.calcsize(">if") == 8; _ledger.append(1)
assert struct.calcsize(">q") == 8; _ledger.append(1)
assert struct.calcsize(">h") == 2; _ledger.append(1)
assert struct.calcsize(">b") == 1; _ledger.append(1)
assert hasattr(struct, "pack") == True; _ledger.append(1)
assert hasattr(struct, "unpack") == True; _ledger.append(1)
assert hasattr(struct, "calcsize") == True; _ledger.append(1)
assert hasattr(struct, "Struct") == True; _ledger.append(1)
assert hasattr(struct, "pack_into") == True; _ledger.append(1)
assert hasattr(struct, "unpack_from") == True; _ledger.append(1)
assert hasattr(struct, "iter_unpack") == True; _ledger.append(1)
assert hasattr(struct, "error") == True; _ledger.append(1)

# 3) typing surface
assert hasattr(typing, "Optional") == True; _ledger.append(1)
assert hasattr(typing, "List") == True; _ledger.append(1)
assert hasattr(typing, "Dict") == True; _ledger.append(1)
assert hasattr(typing, "Tuple") == True; _ledger.append(1)
assert hasattr(typing, "Set") == True; _ledger.append(1)
assert hasattr(typing, "FrozenSet") == True; _ledger.append(1)
assert hasattr(typing, "Callable") == True; _ledger.append(1)
assert hasattr(typing, "Union") == True; _ledger.append(1)
assert hasattr(typing, "Any") == True; _ledger.append(1)
assert hasattr(typing, "ClassVar") == True; _ledger.append(1)
assert hasattr(typing, "Final") == True; _ledger.append(1)
assert hasattr(typing, "Type") == True; _ledger.append(1)
assert hasattr(typing, "TypeVar") == True; _ledger.append(1)
assert hasattr(typing, "Generic") == True; _ledger.append(1)
assert hasattr(typing, "Protocol") == True; _ledger.append(1)
assert hasattr(typing, "cast") == True; _ledger.append(1)
assert hasattr(typing, "get_type_hints") == True; _ledger.append(1)
assert hasattr(typing, "NamedTuple") == True; _ledger.append(1)
assert hasattr(typing, "TypedDict") == True; _ledger.append(1)
assert hasattr(typing, "Iterator") == True; _ledger.append(1)

# 4) weakref surface
assert hasattr(weakref, "ref") == True; _ledger.append(1)
assert hasattr(weakref, "proxy") == True; _ledger.append(1)
assert hasattr(weakref, "WeakValueDictionary") == True; _ledger.append(1)
assert hasattr(weakref, "WeakKeyDictionary") == True; _ledger.append(1)
assert hasattr(weakref, "WeakSet") == True; _ledger.append(1)
assert hasattr(weakref, "finalize") == True; _ledger.append(1)

# 5) abc surface
assert hasattr(abc, "ABC") == True; _ledger.append(1)
assert hasattr(abc, "ABCMeta") == True; _ledger.append(1)
assert hasattr(abc, "abstractmethod") == True; _ledger.append(1)
assert hasattr(abc, "abstractclassmethod") == True; _ledger.append(1)
assert hasattr(abc, "abstractstaticmethod") == True; _ledger.append(1)
assert hasattr(abc, "abstractproperty") == True; _ledger.append(1)

# 6) enum surface (class-level binding only — see spec fixture for value
#    semantics divergence)
assert hasattr(enum, "Enum") == True; _ledger.append(1)
assert hasattr(enum, "IntEnum") == True; _ledger.append(1)
assert hasattr(enum, "Flag") == True; _ledger.append(1)
assert hasattr(enum, "IntFlag") == True; _ledger.append(1)
assert hasattr(enum, "auto") == True; _ledger.append(1)
assert hasattr(enum, "unique") == True; _ledger.append(1)

# 7) dataclasses partial surface
assert hasattr(dataclasses, "dataclass") == True; _ledger.append(1)
assert hasattr(dataclasses, "field") == True; _ledger.append(1)
assert hasattr(dataclasses, "asdict") == True; _ledger.append(1)
assert hasattr(dataclasses, "astuple") == True; _ledger.append(1)
assert hasattr(dataclasses, "fields") == True; _ledger.append(1)

# 8) fractions partial — numerator/denominator + class binding
assert fractions.Fraction(3, 4).numerator == 3; _ledger.append(1)
assert fractions.Fraction(3, 4).denominator == 4; _ledger.append(1)
assert fractions.Fraction(5, 7).numerator == 5; _ledger.append(1)
assert fractions.Fraction(5, 7).denominator == 7; _ledger.append(1)
assert hasattr(fractions, "Fraction") == True; _ledger.append(1)

# 9) decimal partial — class binding only
assert hasattr(decimal, "Decimal") == True; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_json_struct_typing_weakref_abc_value_ops {sum(_ledger)} asserts")
