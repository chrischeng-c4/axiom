# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "572"
# dimension = "surface"
# case = "walrus_parses_and_binds"
# subject = ":="
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
""":=: the walrus operator parses inside a parenthesized condition and binds the name to the assigned value"""

# The PEP's documented language feature parses and runs, binding the name.
if (n := 5) > 0:
    assert n == 5

print("walrus_parses_and_binds OK")
