# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "501"
# dimension = "behavior"
# case = "repr_conversion_applies"
# subject = "fstring.debug_equals"
# kind = "semantic"
# xfail = "mamba strips the `name = ` prefix from PEP 501 debug f-strings (project_mamba_pep_silent_divergences_2026_05_27)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""fstring.debug_equals: an explicit !r conversion after the debug '=' reprs the value: f"{x = !r}" -> "x = 42" with the int repr"""

x = 42
# !r reprs the value (int repr is plain '42'); the 's' below confirms the
# conversion is honored, not the implicit default that the bare '=' would use.
assert f"{x = !r}" == "x = 42", repr(f"{x = !r}")
s = "ab"
assert f"{s = !r}" == "s = 'ab'", repr(f"{s = !r}")
print("repr_conversion_applies OK")
