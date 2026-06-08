# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "498"
# dimension = "behavior"
# case = "field_captures_closure_variable"
# subject = "fstring.scope"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""fstring.scope: a nested function's f-string captures the enclosing variable: closure('987')() is 'x:987' and closure(7)() is 'x:7'"""
# an f-string field closes over enclosing-function variables

def closure(x):
    # A nested function's f-string captures the enclosing variable.
    def inner():
        return f"x:{x}"
    return inner

assert closure("987")() == "x:987"
assert closure(7)() == "x:7"

print("field_captures_closure_variable OK")
