# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "695"
# dimension = "behavior"
# case = "recursive_type_alias_lazy"
# subject = "typing.TypeAliasType"
# kind = "semantic"
# xfail = "`type Recursive = ...` does not bind the alias name on mamba (probed 2026-05-29)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""typing.TypeAliasType: the alias value is lazily evaluated, so type Recursive = Recursive | None may reference itself: __value__ equals Recursive | None"""


# The value is lazily evaluated, so an alias may reference itself recursively.
type Recursive = Recursive | None
assert Recursive.__value__ == Recursive | None

print("recursive_type_alias_lazy OK")
