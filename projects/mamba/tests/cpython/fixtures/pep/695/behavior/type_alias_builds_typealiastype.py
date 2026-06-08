# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "695"
# dimension = "behavior"
# case = "type_alias_builds_typealiastype"
# subject = "typing.TypeAliasType"
# kind = "semantic"
# xfail = "`type X = ...` does not bind the alias name on mamba ('undefined name'; probed 2026-05-29)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""typing.TypeAliasType: type Plain = int builds a TypeAliasType whose __value__ is int, __name__ is 'Plain', and __type_params__ is ()"""


# `type X = ...` builds a TypeAliasType whose target is on __value__.
type Plain = int
assert type(Plain).__name__ == "TypeAliasType"
assert Plain.__value__ is int
assert Plain.__name__ == "Plain"
assert Plain.__type_params__ == ()

print("type_alias_builds_typealiastype OK")
