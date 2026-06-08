# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "572"
# dimension = "behavior"
# case = "default_arg_walrus_allowed"
# subject = ":="
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
""":=: a walrus in a parameter default is allowed (the def does not raise): def f(x=(n := 5)) compiles and f() returns 5"""
# A walrus in a parameter default is allowed; the def does not raise.
def f(x=(n := 5)):
    return x

# The default is evaluated once at def time, binding n in the enclosing scope.
assert n == 5
assert f() == 5
assert f(99) == 99

print("default_arg_walrus_allowed OK")
