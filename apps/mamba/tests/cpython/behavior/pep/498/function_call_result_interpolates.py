# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "498"
# dimension = "behavior"
# case = "function_call_result_interpolates"
# subject = "fstring.expression"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""fstring.expression: a function-call result interpolates: with label(v) returning 'x='+str(v), f'{label(10)}' is 'x=10'"""
# a call expression in a field is evaluated and formatted

def label(v):
    return "x=" + str(v)

assert f"{label(10)}" == "x=10"

print("function_call_result_interpolates OK")
