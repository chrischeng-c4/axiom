# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "572"
# dimension = "behavior"
# case = "keyword_arg_walrus_leaks"
# subject = ":="
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
""":=: a walrus passed as a keyword call argument binds in the surrounding scope after the call"""
# A walrus passed as a keyword argument binds in the surrounding scope.
def identity(value):
    return value

out2 = identity(value=(kw := 7))
assert out2 == 7
assert kw == 7

print("keyword_arg_walrus_leaks OK")
