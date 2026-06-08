# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "572"
# dimension = "behavior"
# case = "walrus_in_if_guard_binds"
# subject = ":="
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
""":=: a walrus in an if-condition binds the name and the bound value is visible in the body: if (n := 10) > 5 sees n == 10"""
# Walrus binds + returns the value; the binding is visible in the body.
taken = False
if (n := 10) > 5:
    taken = True
    assert n == 10
assert taken is True
assert n == 10

print("walrus_in_if_guard_binds OK")
