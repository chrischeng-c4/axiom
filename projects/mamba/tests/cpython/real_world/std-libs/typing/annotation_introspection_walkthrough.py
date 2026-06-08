# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "typing"
# dimension = "real_world"
# case = "annotation_introspection_walkthrough"
# subject = "typing"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_typing.py"
# status = "filled"
# ///
"""typing: a downstream tool annotates a dataclass-style function with List/Dict/Optional/Union, then drives get_type_hints + get_origin + get_args to resolve and decompose every annotation into its runtime origin and parameters"""
import typing


def handler(
    items: typing.List[int],
    index: typing.Dict[str, int],
    note: typing.Optional[str],
    value: typing.Union[int, str],
) -> bool:
    return bool(items) and bool(index) and note is not None and value is not None


# 1. Resolve every annotation in one pass.
hints = typing.get_type_hints(handler)
assert set(hints) == {"items", "index", "note", "value", "return"}, f"hint keys = {sorted(hints)!r}"
assert hints["return"] is bool, "return annotation resolves to bool"

# 2. Decompose List[int].
assert typing.get_origin(hints["items"]) is list, "items origin is list"
assert typing.get_args(hints["items"]) == (int,), "items args are (int,)"

# 3. Decompose Dict[str, int].
assert typing.get_origin(hints["index"]) is dict, "index origin is dict"
assert typing.get_args(hints["index"]) == (str, int), "index args are (str, int)"

# 4. Optional[str] is Union[str, None].
assert typing.get_origin(hints["note"]) is typing.Union, "note origin is Union"
assert typing.get_args(hints["note"]) == (str, type(None)), "note args are (str, NoneType)"

# 5. Union[int, str].
assert typing.get_origin(hints["value"]) is typing.Union, "value origin is Union"
assert typing.get_args(hints["value"]) == (int, str), "value args are (int, str)"

print("annotation_introspection_walkthrough OK")
