# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "percent_numeric_handles"
# dimension = "behavior"
# case = "str_percent_mapping_control"
# subject = "str.percent_format"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""str.percent_format: ordinary string mapping percent formatting remains green"""
assert "%(name)s=%(value)d" % {"name": "ok", "value": 7} == "ok=7"

print("str_percent_mapping_control OK")
