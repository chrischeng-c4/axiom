# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "percent_numeric_handles"
# dimension = "behavior"
# case = "str_percent_tuple_control"
# subject = "str.percent_format"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""str.percent_format: ordinary string tuple percent formatting remains green"""
assert "value=%d name=%s" % (7, "ok") == "value=7 name=ok"

print("str_percent_tuple_control OK")
