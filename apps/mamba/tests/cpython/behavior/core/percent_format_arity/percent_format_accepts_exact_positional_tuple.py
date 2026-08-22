# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "percent_format_arity"
# dimension = "behavior"
# case = "percent_format_accepts_exact_positional_tuple"
# subject = "str.percent_format"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""str.percent_format: an exact two-item positional tuple fills two percent s slots."""
# formatting syntax uses only Python built-ins

assert "%s %s" % ("Ada", "Lovelace") == "Ada Lovelace"

print("percent_format_accepts_exact_positional_tuple OK")
