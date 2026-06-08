# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "501"
# dimension = "surface"
# case = "debug_equals_form_parses"
# subject = "f'{(10) = }'"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""f'{(10) = }': debug_equals_form_parses (surface)."""
pass

assert type(f'{(10) = }').__name__ == "str"
print("debug_equals_form_parses OK")
