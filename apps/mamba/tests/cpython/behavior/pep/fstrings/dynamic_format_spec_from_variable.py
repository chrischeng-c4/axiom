# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "fstrings"
# dimension = "behavior"
# case = "dynamic_format_spec_from_variable"
# subject = "fstring.format_spec"
# kind = "semantic"
# xfail = "mamba diverges on a fully dynamic format spec built from a variable (retired behavior.py head comment: AssertionError dynamic fmt = '3.14159265'; project_mamba_pep_silent_divergences_2026_05_27)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""fstring.format_spec: the whole format spec is itself an expression: with fmt='.3f', pi=3.14159265, f'{pi:{fmt}}' is '3.142'"""
# the format spec after ':' is evaluated as an expression

fmt = ".3f"
pi = 3.14159265
result = f"{pi:{fmt}}"
assert result == "3.142", f"dynamic fmt = {result!r}"

print("dynamic_format_spec_from_variable OK")
