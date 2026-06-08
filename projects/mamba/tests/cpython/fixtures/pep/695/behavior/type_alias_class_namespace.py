# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "695"
# dimension = "behavior"
# case = "type_alias_class_namespace"
# subject = "typing.TypeAliasType"
# kind = "semantic"
# xfail = "`type Inner = ...` inside a class body does not bind on mamba (probed 2026-05-29)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""typing.TypeAliasType: inside a class body an alias resolves names against the class namespace: class Holder with member=int and type Inner=member gives Holder.Inner.__value__ is int"""


# Inside a class body an alias resolves names against the class namespace.
class Holder:
    member = int
    type Inner = member


assert Holder.Inner.__value__ is int

print("type_alias_class_namespace OK")
