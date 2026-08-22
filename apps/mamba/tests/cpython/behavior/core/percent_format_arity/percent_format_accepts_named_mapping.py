# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "percent_format_arity"
# dimension = "behavior"
# case = "percent_format_accepts_named_mapping"
# subject = "str.percent_format"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""str.percent_format: a named percent s format consumes a one-key mapping."""
# formatting syntax uses only Python built-ins

assert "%(name)s" % {"name": "Ada"} == "Ada"

print("percent_format_accepts_named_mapping OK")
