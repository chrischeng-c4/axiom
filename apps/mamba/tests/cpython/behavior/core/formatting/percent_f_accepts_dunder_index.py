# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "formatting"
# dimension = "behavior"
# case = "percent_f_accepts_dunder_index"
# subject = "str.percent_format"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""str.percent_format: percent f formatting uses a custom __index__ method returning 7"""
# formatting syntax uses only Python built-ins

class IndexOnly:
    def __index__(self):
        return 7


obj = IndexOnly()
assert "%.3f" % obj == "7.000"

print("percent_f_accepts_dunder_index OK")
