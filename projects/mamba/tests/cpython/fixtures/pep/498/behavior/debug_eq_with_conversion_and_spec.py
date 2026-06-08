# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "498"
# dimension = "behavior"
# case = "debug_eq_with_conversion_and_spec"
# subject = "fstring.debug"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""fstring.debug: an explicit conversion or spec overrides the =-debug default repr: with val=3.14159, f'{val=:.2f}' is 'val=3.14' and f'{name=!s}' is 'name=world' for name='world'"""
# !s or a format spec override the default repr in a debug field

val = 3.14159
name = "world"
assert f"{val=:.2f}" == "val=3.14"
assert f"{name=!s}" == "name=world"

print("debug_eq_with_conversion_and_spec OK")
