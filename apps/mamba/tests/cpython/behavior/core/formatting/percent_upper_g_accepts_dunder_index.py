# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "formatting"
# dimension = "behavior"
# case = "percent_upper_g_accepts_dunder_index"
# subject = "str.percent_format"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""str.percent_format: percent upper G formatting uses __index__ returning 7"""
class IndexOnly:
    def __index__(self):
        return 7


assert "%.3G" % IndexOnly() == "7"

print("percent_upper_g_accepts_dunder_index OK")
