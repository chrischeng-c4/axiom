# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "fstrings"
# dimension = "behavior"
# case = "method_call_in_field"
# subject = "fstring.expression"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""fstring.expression: a method call inside a field is evaluated and formatted: f"{'hello'.upper()}" is 'HELLO'"""
# a call expression in a field is evaluated and formatted

assert f"{'hello'.upper()}" == "HELLO", "method call in f-string"

print("method_call_in_field OK")
