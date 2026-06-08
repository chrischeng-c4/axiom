# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "498"
# dimension = "behavior"
# case = "field_sees_locals_and_shadowing"
# subject = "fstring.scope"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""fstring.scope: a field reads locals and a local shadows the same-named global within the function"""
# a local binding shadows a global of the same name in a field

a_global = "global variable"

def uses_local():
    a_local = "local variable"
    return f"g:{a_global} l:{a_local}"

def shadows_global():
    # A local of the same name shadows the global within the function.
    a_global = "really a local"
    return f"g:{a_global!r}"

assert uses_local() == "g:global variable l:local variable"
assert shadows_global() == "g:'really a local'"

print("field_sees_locals_and_shadowing OK")
