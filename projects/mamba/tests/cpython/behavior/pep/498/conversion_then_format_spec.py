# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "498"
# dimension = "behavior"
# case = "conversion_then_format_spec"
# subject = "fstring.conversion"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""fstring.conversion: a conversion then a format spec apply in order: f'{3 != 4!s:.3}' converts True via str then truncates to 'Tru'"""
# conversion runs before the format spec

assert f"{3 != 4!s:.3}" == "Tru"

print("conversion_then_format_spec OK")
