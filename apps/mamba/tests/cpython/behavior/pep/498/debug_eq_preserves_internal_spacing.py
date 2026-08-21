# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "498"
# dimension = "behavior"
# case = "debug_eq_preserves_internal_spacing"
# subject = "fstring.debug"
# kind = "semantic"
# xfail = "mamba strips the expression name/spacing from the {expr = } debug form (project_mamba_pep_silent_divergences_2026_05_27)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""fstring.debug: the =-debug form echoes verbatim spacing/operators: f'{x = }' is 'x = 10', f'1 == 2={1 == 2!r}' is '1 == 2=False', f'{1 + 2 = }' is '1 + 2 = 3', f'{total*2=}' is 'total*2=6'"""
# whitespace and operators inside the debug field are echoed verbatim

x = 10
# Surrounding spaces around `=` are preserved verbatim in the text.
assert f"{x = }" == "x = 10"
# The echoed expression text is the verbatim source between the braces,
# including any internal whitespace and operators (gh-129093).
assert f"1==2={1 == 2!r}" == "1==2=False"
assert f"1 == 2={1 == 2!r}" == "1 == 2=False"
assert f"1!=2={1 != 2!r}" == "1!=2=True"
assert f"1 != 2={1 != 2!r}" == "1 != 2=True"
assert f"(1*2) != (3)={1 * 2 != 3!r}" == "(1*2) != (3)=True"
total = 1 + 2
assert f"{1 + 2 = }" == "1 + 2 = 3"
assert f"{total*2=}" == "total*2=6"

print("debug_eq_preserves_internal_spacing OK")
