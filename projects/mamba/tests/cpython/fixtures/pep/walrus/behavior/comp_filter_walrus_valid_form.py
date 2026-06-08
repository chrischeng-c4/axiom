# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "walrus"
# dimension = "behavior"
# case = "comp_filter_walrus_valid_form"
# subject = ":="
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
""":=: a walrus bound in a comprehension filter is usable as the element expression: [d for v in range(5) if (d := v*2) > 2] yields [4, 6, 8]"""
# A filter-bound walrus is usable as the element expression.
valid = [doubled for v in range(5) if (doubled := v * 2) > 2]
assert valid == [4, 6, 8], f"valid = {valid!r}"

print("comp_filter_walrus_valid_form OK")
