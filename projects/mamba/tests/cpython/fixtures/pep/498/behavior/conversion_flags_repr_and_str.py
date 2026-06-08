# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "498"
# dimension = "behavior"
# case = "conversion_flags_repr_and_str"
# subject = "fstring.conversion"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""fstring.conversion: !r uses repr and !s uses str: f"{'x'!r}" is "'x'" and f'{3 != 4!s}' is 'True'"""
# !r / !s conversion flags select repr / str before formatting

assert f"{'x'!r}" == "'x'"
assert f"{3 != 4!s}" == "True"

print("conversion_flags_repr_and_str OK")
