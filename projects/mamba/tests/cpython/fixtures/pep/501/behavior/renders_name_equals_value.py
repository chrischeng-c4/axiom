# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "501"
# dimension = "behavior"
# case = "renders_name_equals_value"
# subject = "fstring.debug_equals"
# kind = "semantic"
# xfail = "mamba strips the `name = ` prefix from PEP 501 debug f-strings, rendering just the value (project_mamba_pep_silent_divergences_2026_05_27)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""fstring.debug_equals: f"{x = }" renders the expression source, an '=', and the value -> 'x = 10'"""

x = 10
assert f"{x = }" == "x = 10", repr(f"{x = }")
print("renders_name_equals_value OK")
