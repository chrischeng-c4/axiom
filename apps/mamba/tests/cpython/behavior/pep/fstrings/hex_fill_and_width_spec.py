# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "fstrings"
# dimension = "behavior"
# case = "hex_fill_and_width_spec"
# subject = "fstring.format_spec"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""fstring.format_spec: an alternate-form zero-padded hex spec applies fill and width: f'{255:#010x}' is '0x000000ff'"""
# '#' alt-form, zero fill, and width compose in the format spec

assert f"{255:#010x}" == "0x000000ff", f"hex fmt = {f'{255:#010x}'!r}"

print("hex_fill_and_width_spec OK")
