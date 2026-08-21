# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "501"
# dimension = "behavior"
# case = "whitespace_around_equals_preserved"
# subject = "fstring.debug_equals"
# kind = "semantic"
# xfail = "mamba strips the `name = ` prefix from PEP 501 debug f-strings (project_mamba_pep_silent_divergences_2026_05_27)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""fstring.debug_equals: leading/trailing inner whitespace is preserved verbatim: f"{x= }" -> 'x= 10' and f"{x =}" -> 'x =10'"""

x = 10
# The text between the expression and the '=' is echoed verbatim; only one
# trailing space is implied by the default conversion when none is given.
assert f"{x= }" == "x= 10", repr(f"{x= }")
assert f"{x =}" == "x =10", repr(f"{x =}")
print("whitespace_around_equals_preserved OK")
