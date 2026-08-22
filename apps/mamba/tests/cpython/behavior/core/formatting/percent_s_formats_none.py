# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "formatting"
# dimension = "behavior"
# case = "percent_s_formats_none"
# subject = "str.percent_format"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""str.percent_format: percent s formatting renders None using its deterministic string representation"""
# formatting syntax uses only Python built-ins

assert "%s" % None == "None"

print("percent_s_formats_none OK")
