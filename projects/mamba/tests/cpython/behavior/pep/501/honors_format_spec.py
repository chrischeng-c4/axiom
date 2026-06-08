# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "501"
# dimension = "behavior"
# case = "honors_format_spec"
# subject = "fstring.debug_equals"
# kind = "semantic"
# xfail = "mamba strips the `name = ` prefix from PEP 501 debug f-strings (project_mamba_pep_silent_divergences_2026_05_27)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""fstring.debug_equals: a format spec after the debug '=' applies to the value only: f"{x = :04d}" -> 'x = 0042'"""

x = 42
assert f"{x = :04d}" == "x = 0042", repr(f"{x = :04d}")
print("honors_format_spec OK")
