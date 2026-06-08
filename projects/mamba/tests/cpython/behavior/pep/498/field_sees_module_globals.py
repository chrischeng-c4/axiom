# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "498"
# dimension = "behavior"
# case = "field_sees_module_globals"
# subject = "fstring.scope"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""fstring.scope: an f-string field reads module globals: a function returning f'g:{a_global}' sees the module-level a_global"""
# replacement fields resolve names in the enclosing scope chain

a_global = "global variable"

def uses_global():
    # An f-string sees module globals.
    return f"g:{a_global}"

assert uses_global() == "g:global variable"

print("field_sees_module_globals OK")
