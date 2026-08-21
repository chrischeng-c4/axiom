# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "695"
# dimension = "behavior"
# case = "type_alias_lazy_classvar_mutation"
# subject = "typing.TypeAliasType"
# kind = "semantic"
# xfail = "`type Alias = ...` inside a class body does not bind on mamba (probed 2026-05-29)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""typing.TypeAliasType: lazy evaluation observes a class-var change before first access: rebinding Mutable.seed to float before touching the alias makes Mutable.Alias.__value__ float"""


# Lazy evaluation means a class-var change before first access is observed.
class Mutable:
    seed = int
    type Alias = seed


Mutable.seed = float
assert Mutable.Alias.__value__ is float

print("type_alias_lazy_classvar_mutation OK")
