# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "fstrings"
# dimension = "behavior"
# case = "ascii_conversion_escapes_non_ascii"
# subject = "fstring.conversion"
# kind = "semantic"
# xfail = "mamba diverges on the !a ASCII conversion of a non-ASCII string (retired surface.py head comment: AssertionError !a = \"'cafe'\"; project_mamba_pep_silent_divergences_2026_05_27)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""fstring.conversion: !a escapes non-ASCII characters via ascii(): f'{"caf\\u00e9"!a}' contains the backslash-escaped code point '\\\\u00e9'"""
# !a converts via ascii(), backslash-escaping non-ASCII code points

unicode_str = "caf\u00e9"
ascii_result = f"{unicode_str!a}"
assert "\\u00e9" in ascii_result or "\\xe9" in ascii_result, f"!a = {ascii_result!r}"

print("ascii_conversion_escapes_non_ascii OK")
