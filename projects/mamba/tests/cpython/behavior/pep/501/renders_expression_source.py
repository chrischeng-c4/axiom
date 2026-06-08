# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "501"
# dimension = "behavior"
# case = "renders_expression_source"
# subject = "fstring.debug_equals"
# kind = "semantic"
# xfail = "mamba strips the `expr = ` prefix from PEP 501 debug f-strings (project_mamba_pep_silent_divergences_2026_05_27)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""fstring.debug_equals: f"{x + 1 = }" echoes the full expression source text before the '=' -> 'x + 1 = 4'"""

x = 3
assert f"{x + 1 = }" == "x + 1 = 4", repr(f"{x + 1 = }")
print("renders_expression_source OK")
