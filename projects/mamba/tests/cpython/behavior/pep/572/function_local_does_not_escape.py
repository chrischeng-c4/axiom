# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "572"
# dimension = "behavior"
# case = "function_local_does_not_escape"
# subject = ":="
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
""":=: a walrus target inside a function is local and does not escape to the module after the function returns"""
# A walrus target inside a function is local; it does not escape to the
# module after the function returns.
def assign_inside():
    (secret := 5)
    return secret

assert assign_inside() == 5
assert "secret" not in globals()

print("function_local_does_not_escape OK")
