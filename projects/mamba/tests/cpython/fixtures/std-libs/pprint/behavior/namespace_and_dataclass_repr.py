# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pprint"
# dimension = "behavior"
# case = "namespace_and_dataclass_repr"
# subject = "pprint.pformat"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_pprint.py"
# status = "filled"
# ///
"""pprint.pformat: SimpleNamespace renders as namespace(field=value,...) (subclass uses its own class name) and a dataclass renders as ClassName(field=value,...); both wrap and align fields to the width"""
import dataclasses
import types

import pprint

# A small namespace stays on one line.
assert pprint.pformat(types.SimpleNamespace(a=1, b=2)) == "namespace(a=1, b=2)"

# Empty namespace.
assert pprint.pformat(types.SimpleNamespace()) == "namespace()"

# A wide namespace wraps one field per line, aligned under the prefix.
ns = types.SimpleNamespace(the=0, quick=1, brown=2)
assert pprint.pformat(ns, width=20, indent=4) == (
    "namespace(the=0,\n          quick=1,\n          brown=2)"
)


# A SimpleNamespace subclass uses its own class name as the prefix.
class AdvancedNamespace(types.SimpleNamespace):
    pass


adv = AdvancedNamespace(x=1, y=2)
assert pprint.pformat(adv) == "AdvancedNamespace(x=1, y=2)"


# Dataclass instances render as ClassName(field=value, ...).
@dataclasses.dataclass
class Point:
    x: int
    y: int


assert pprint.pformat(Point(1, 2)) == "Point(x=1, y=2)"

# An empty dataclass renders with empty parens.
Empty = dataclasses.make_dataclass("Empty", ())
assert pprint.pformat(Empty()) == "Empty()"
print("namespace_and_dataclass_repr OK")
