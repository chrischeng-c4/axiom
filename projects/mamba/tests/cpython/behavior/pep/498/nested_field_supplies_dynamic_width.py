# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "498"
# dimension = "behavior"
# case = "nested_field_supplies_dynamic_width"
# subject = "fstring.format_spec"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""fstring.format_spec: a nested {} inside the spec supplies the width dynamically: with width=10, f"x={'foo'*2:{width}}" is 'x=foofoo    ' and f'{10*2:{width}}' is '        20'"""
# a replacement field may appear inside the format spec

y = 2
width = 10
assert f"x={'foo' * y:{width}}" == "x=foofoo    "
assert f"{10 * y:{width}}" == "        20"

print("nested_field_supplies_dynamic_width OK")
