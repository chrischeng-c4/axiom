# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "695"
# dimension = "behavior"
# case = "parameterized_type_alias"
# subject = "typing.TypeAliasType"
# kind = "semantic"
# xfail = "`type Pair[T] = ...` does not bind the alias name on mamba (probed 2026-05-29)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""typing.TypeAliasType: a parameterized alias type Pair[T] = list[T] keeps its own __type_params__ and its __value__ equals list[T]"""


# A parameterized alias keeps its own type params.
type Pair[T] = list[T]
t_param, = Pair.__type_params__
assert Pair.__value__ == list[t_param]
assert t_param.__name__ == "T"

print("parameterized_type_alias OK")
