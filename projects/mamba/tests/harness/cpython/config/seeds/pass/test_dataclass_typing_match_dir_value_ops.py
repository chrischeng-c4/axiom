# Operational AssertionPass seed for the value contract of five
# bootstrap stdlib / language surfaces used by every modeling /
# pattern-matching / introspection / printf-formatting path:
# `dataclasses` (the documented top-level `dataclass` / `asdict`
# / `astuple` / `fields` / `field` helper attribute surface),
# `typing` (the documented TYPE_CHECKING constant + the
# documented `TypedDict` / `NamedTuple` / `Protocol` / `Generic`
# / `TypeVar` / `cast` / `Optional` / `Union` / `List` / `Dict` /
# `Tuple` / `Callable` attribute surface), Python's `match` /
# `case` statement (the documented literal / guard / list /
# dict / wildcard patterns), the printf-style `%` operator on
# strings, and the introspection sextet (`hasattr` / `getattr`
# / `setattr` / `delattr` / `vars` / `dir` / `globals` /
# `locals`).
#
# The matching subset between mamba and CPython is the
# attribute-surface layer + `match` / `case` literal-guard-
# list-dict-wildcard pattern layer + printf-style integer /
# string / float / hex / oct / tuple-dispatch / named-dict-
# dispatch layer + attribute reflection layer + eval-simple
# layer.
#
# Surface in this fixture:
#   • dataclasses — top-level attribute surface (`dataclass`,
#     `asdict`, `astuple`, `fields`, `field`);
#   • typing — TYPE_CHECKING constant + bare attribute surface
#     (`TypedDict`, `NamedTuple`, `Protocol`, `Generic`,
#     `TypeVar`, `cast`, `Optional`, `Union`, `List`, `Dict`,
#     `Tuple`, `Callable`);
#   • match / case — literal pattern, guard pattern, list
#     pattern, dict capture, wildcard;
#   • string `%` operator — `%d`, `%s`, `%.2f`, multi-arg
#     tuple, padded `%5d`, zero-padded `%05d`, `%x` / `%X` /
#     `%o`, named-dict `%(name)s`, `%r`;
#   • introspection — hasattr / getattr / setattr / delattr /
#     vars / dir / globals / locals + callable on lambda &
#     non-callable list + eval of arithmetic literal.
#
# Behavioral edges that DIVERGE on mamba (dataclasses.replace /
# Field / FrozenInstanceError / is_dataclass / MISSING absent
# from the documented attribute surface, typing.Any repr
# diverging from "typing.Any", typing.TypedDict instance
# construction returning empty + isinstance(dict) False,
# typing.NamedTuple instance construction returning empty +
# attribute access returning None, "%e" % 1e6 returning the
# literal "%e" instead of expanded scientific notation,
# callable(1) returning True (int callable), eval("x", globals)
# returning None when locals/globals arg supplied,
# exec("y=100", ns) not populating ns, compile + eval(code,
# globals) returning None) are covered in the matching spec
# fixture `lang_dataclass_typing_eval_silent`.
import dataclasses
import typing


_ledger: list[int] = []


def _classify(x: int) -> str:
    match x:
        case 0:
            return "zero"
        case n if n > 0:
            return "positive"
        case _:
            return "negative"


def _shape(obj: object) -> str:
    match obj:
        case [1, 2]:
            return "pair12"
        case [_, _, _]:
            return "triple"
        case {"name": _}:
            return "named"
        case _:
            return "other"


class _PlainC:
    cls_attr = 100

    def __init__(self) -> None:
        self.inst_attr = 50


# 1) dataclasses — top-level attribute surface
assert hasattr(dataclasses, "dataclass") == True; _ledger.append(1)
assert hasattr(dataclasses, "asdict") == True; _ledger.append(1)
assert hasattr(dataclasses, "astuple") == True; _ledger.append(1)
assert hasattr(dataclasses, "fields") == True; _ledger.append(1)
assert hasattr(dataclasses, "field") == True; _ledger.append(1)

# 2) typing — TYPE_CHECKING constant + bare attribute surface
assert typing.TYPE_CHECKING == False; _ledger.append(1)
assert hasattr(typing, "TypedDict") == True; _ledger.append(1)
assert hasattr(typing, "NamedTuple") == True; _ledger.append(1)
assert hasattr(typing, "Protocol") == True; _ledger.append(1)
assert hasattr(typing, "Generic") == True; _ledger.append(1)
assert hasattr(typing, "TypeVar") == True; _ledger.append(1)
assert hasattr(typing, "cast") == True; _ledger.append(1)
assert hasattr(typing, "Optional") == True; _ledger.append(1)
assert hasattr(typing, "Union") == True; _ledger.append(1)
assert hasattr(typing, "List") == True; _ledger.append(1)
assert hasattr(typing, "Dict") == True; _ledger.append(1)
assert hasattr(typing, "Tuple") == True; _ledger.append(1)
assert hasattr(typing, "Callable") == True; _ledger.append(1)
assert hasattr(typing, "Any") == True; _ledger.append(1)

# 3) match / case — literal, guard, list, dict, wildcard
assert _classify(0) == "zero"; _ledger.append(1)
assert _classify(5) == "positive"; _ledger.append(1)
assert _classify(-1) == "negative"; _ledger.append(1)
assert _shape([1, 2]) == "pair12"; _ledger.append(1)
assert _shape([1, 2, 3]) == "triple"; _ledger.append(1)
assert _shape({"name": "alice"}) == "named"; _ledger.append(1)
assert _shape(42) == "other"; _ledger.append(1)

# 4) string % formatting
assert "%d" % 42 == "42"; _ledger.append(1)
assert "%s" % "hello" == "hello"; _ledger.append(1)
assert "%.2f" % 3.14159 == "3.14"; _ledger.append(1)
assert "%s=%d" % ("x", 42) == "x=42"; _ledger.append(1)
assert "%5d" % 42 == "   42"; _ledger.append(1)
assert "%05d" % 42 == "00042"; _ledger.append(1)
assert "%x" % 255 == "ff"; _ledger.append(1)
assert "%X" % 255 == "FF"; _ledger.append(1)
assert "%o" % 8 == "10"; _ledger.append(1)
assert "%(name)s=%(age)d" % {"name": "a", "age": 30} == "a=30"; _ledger.append(1)
assert "%r" % "x" == "'x'"; _ledger.append(1)

# 5) Introspection — hasattr / getattr / setattr / delattr / vars / dir
_c = _PlainC()
assert hasattr(_c, "inst_attr") == True; _ledger.append(1)
assert hasattr(_c, "cls_attr") == True; _ledger.append(1)
assert hasattr(_c, "missing") == False; _ledger.append(1)
assert getattr(_c, "inst_attr") == 50; _ledger.append(1)
assert getattr(_c, "missing", "DEF") == "DEF"; _ledger.append(1)
setattr(_c, "new_attr", 999)
assert getattr(_c, "new_attr") == 999; _ledger.append(1)
delattr(_c, "new_attr")
assert hasattr(_c, "new_attr") == False; _ledger.append(1)
assert "inst_attr" in vars(_c); _ledger.append(1)
assert "inst_attr" in dir(_c); _ledger.append(1)
assert "cls_attr" in dir(_c); _ledger.append(1)
assert isinstance(globals(), dict); _ledger.append(1)
assert isinstance(locals(), dict); _ledger.append(1)

# 6) callable + eval simple
assert callable(lambda x: x) == True; _ledger.append(1)
assert callable(str) == True; _ledger.append(1)
assert callable([1, 2]) == False; _ledger.append(1)
assert eval("1+2") == 3; _ledger.append(1)

# NB: dataclasses.replace / Field / FrozenInstanceError /
# is_dataclass / MISSING absent from the documented attribute
# surface, typing.Any repr diverging from "typing.Any",
# typing.TypedDict instance construction returning empty +
# isinstance(dict) False, typing.NamedTuple instance
# construction returning empty + attribute access returning
# None, "%e" % 1e6 returning the literal "%e" instead of
# expanded scientific notation, callable(1) returning True,
# eval("x", globals) returning None when locals/globals arg
# supplied, exec("y=100", ns) not populating ns, compile +
# eval(code, globals) returning None all DIVERGE on mamba —
# moved to the divergence-spec fixture.

print(f"MAMBA_ASSERTION_PASS: test_dataclass_typing_match_dir_value_ops {sum(_ledger)} asserts")
