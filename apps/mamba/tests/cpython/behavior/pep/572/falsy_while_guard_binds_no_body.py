# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "572"
# dimension = "behavior"
# case = "falsy_while_guard_binds_no_body"
# subject = ":="
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
""":=: a walrus while-guard that is falsy never runs the body but still binds the name: while (flag := False) leaves flag False and the body unexecuted"""
# Walrus loop guard that is falsy never runs the body but still binds.
ran = False
while (flag := False):
    ran = True
assert ran is False
assert flag is False

print("falsy_while_guard_binds_no_body OK")
