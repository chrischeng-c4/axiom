# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
"""dict methods: documented exception paths (CPython 3.12 oracle)."""


# Missing key raises KeyError.
try:
    {}["missing"]
    print("missing_key: no_raise")
except KeyError as e:
    print("missing_key:", type(e).__name__, str(e)[:60])


# pop on missing key raises KeyError.
try:
    {}.pop("missing")
    print("pop_missing: no_raise")
except KeyError as e:
    print("pop_missing:", type(e).__name__, str(e)[:60])


# popitem on empty raises KeyError.
try:
    {}.popitem()
    print("popitem_empty: no_raise")
except KeyError as e:
    print("popitem_empty:", type(e).__name__, str(e)[:60])


# Unhashable key raises TypeError.
try:
    {[1, 2]: "value"}  # type: ignore[misc]
    print("list_key: no_raise")
except TypeError as e:
    print("list_key:", type(e).__name__, str(e)[:60])


# Update with non-mapping/non-iterable raises TypeError.
try:
    {}.update(42)  # type: ignore[arg-type]
    print("update_int: no_raise")
except TypeError as e:
    print("update_int:", type(e).__name__, str(e)[:60])


# fromkeys with non-iterable raises TypeError.
try:
    dict.fromkeys(42)  # type: ignore[arg-type]
    print("fromkeys_int: no_raise")
except TypeError as e:
    print("fromkeys_int:", type(e).__name__, str(e)[:60])


# Adding a dict to int raises TypeError.
try:
    {} + 1  # type: ignore[operator]
    print("dict_plus_int: no_raise")
except TypeError as e:
    print("dict_plus_int:", type(e).__name__, str(e)[:60])


# dict views: keys() & list returns view-set; & non-set-like
# may not match; check that operations don't crash for valid inputs.
d = {"a": 1, "b": 2}
print("keys_set_op:", d.keys() & {"a", "c"})


# View set operations with a non-iterable right operand raise TypeError.
for op_name in ("and", "or", "xor", "sub"):
    try:
        if op_name == "and":
            {}.keys() & 1  # type: ignore[operator]
        elif op_name == "or":
            {}.keys() | 1  # type: ignore[operator]
        elif op_name == "xor":
            {}.keys() ^ 1  # type: ignore[operator]
        else:
            {}.items() - 1  # type: ignore[operator]
        print(f"view_{op_name}_int: no_raise")
    except TypeError as e:
        print(f"view_{op_name}_int:", type(e).__name__)


# update with a malformed pair (wrong arity) raises ValueError.
try:
    {}.update([(1, 2, 3)])  # type: ignore[list-item]
    print("update_bad_pair: no_raise")
except ValueError as e:
    print("update_bad_pair:", type(e).__name__)


# dict(**mapping) requires string keyword keys; int keys raise TypeError.
try:
    dict(**{1: 2})  # type: ignore[arg-type]
    print("kwargs_int_key: no_raise")
except TypeError as e:
    print("kwargs_int_key:", type(e).__name__)


# Zero-arg calls to methods that require an argument raise TypeError.
for label, call in (
    ("setdefault", lambda: {}.setdefault()),       # type: ignore[call-arg]
    ("pop", lambda: {}.pop()),                      # type: ignore[call-arg]
    ("getitem", lambda: {}.__getitem__()),          # type: ignore[call-arg]
    ("fromkeys", lambda: dict.fromkeys()),          # type: ignore[call-arg]
):
    try:
        call()
        print(f"{label}_noarg: no_raise")
    except TypeError as e:
        print(f"{label}_noarg:", type(e).__name__)


# View object types are not directly constructible.
for label, vtype in (
    ("keys", type({}.keys())),
    ("values", type({}.values())),
    ("items", type({}.items())),
):
    try:
        vtype()
        print(f"{label}_type_call: no_raise")
    except TypeError as e:
        print(f"{label}_type_call:", type(e).__name__)
