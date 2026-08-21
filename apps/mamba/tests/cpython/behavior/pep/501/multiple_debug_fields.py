# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "501"
# dimension = "behavior"
# case = "multiple_debug_fields"
# subject = "fstring.debug_equals"
# kind = "semantic"
# xfail = "mamba strips the `name = ` prefix from PEP 501 debug f-strings (project_mamba_pep_silent_divergences_2026_05_27)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""fstring.debug_equals: two `{expr = }` fields in one f-string each render their own 'name = value' segment -> "x = 42, y = 'hi'" """

x = 42
y = "hi"
assert f"{x = }, {y = }" == "x = 42, y = 'hi'", repr(f"{x = }, {y = }")
print("multiple_debug_fields OK")
