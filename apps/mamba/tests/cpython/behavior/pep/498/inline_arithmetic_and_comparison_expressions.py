# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "498"
# dimension = "behavior"
# case = "inline_arithmetic_and_comparison_expressions"
# subject = "fstring.expression"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""fstring.expression: arbitrary expressions evaluate inline: f'{x*2}' is '20' (x=10), f'{3 + 4}' is '7', f'{0 == 1}' is 'False', f'{3 != 4}' is 'True'"""
# replacement fields hold arbitrary expressions

x = 10
assert f"{x*2}" == "20"
assert f"{3 + 4}" == "7"
assert f"{0 == 1}" == "False"
assert f"{3 != 4}" == "True"

print("inline_arithmetic_and_comparison_expressions OK")
