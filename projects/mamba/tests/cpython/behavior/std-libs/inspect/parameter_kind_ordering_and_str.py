# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "inspect"
# dimension = "behavior"
# case = "parameter_kind_ordering_and_str"
# subject = "inspect.Parameter"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""inspect.Parameter: Parameter kinds have a defined ordering POSITIONAL_ONLY < POSITIONAL_OR_KEYWORD < VAR_POSITIONAL < KEYWORD_ONLY < VAR_KEYWORD and a readable str form"""
import inspect

P = inspect.Parameter

assert P.POSITIONAL_ONLY < P.POSITIONAL_OR_KEYWORD < P.VAR_POSITIONAL, "kind order lo"
assert P.VAR_POSITIONAL < P.KEYWORD_ONLY < P.VAR_KEYWORD, "kind order hi"
assert str(P.POSITIONAL_ONLY) == "POSITIONAL_ONLY", "kind str"

print("parameter_kind_ordering_and_str OK")
