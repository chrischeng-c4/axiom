# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "572"
# dimension = "behavior"
# case = "call_arg_walrus_leaks"
# subject = ":="
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
""":=: a walrus passed as a positional call argument binds in the surrounding scope after the call"""
# A walrus passed as a call argument binds in the surrounding scope.
def identity(value):
    return value

out = identity((arg := 2))
assert out == 2
assert arg == 2

print("call_arg_walrus_leaks OK")
