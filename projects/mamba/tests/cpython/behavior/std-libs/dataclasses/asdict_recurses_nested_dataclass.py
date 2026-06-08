# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dataclasses"
# dimension = "behavior"
# case = "asdict_recurses_nested_dataclass"
# subject = "dataclasses.asdict"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_dataclasses.py"
# status = "filled"
# ///
"""dataclasses.asdict: asdict() recursively converts a nested dataclass field into a nested dict, preserving leaf values"""
import dataclasses


@dataclasses.dataclass
class Inner:
    val: int


@dataclasses.dataclass
class Outer:
    inner: Inner
    name: str


outer = Outer(Inner(42), "test")
d = dataclasses.asdict(outer)
assert isinstance(d["inner"], dict), f"nested asdict = {type(d['inner'])!r}"
assert d["inner"]["val"] == 42, f"nested val = {d['inner']['val']!r}"
assert d["name"] == "test", f"outer name = {d['name']!r}"

print("asdict_recurses_nested_dataclass OK")
