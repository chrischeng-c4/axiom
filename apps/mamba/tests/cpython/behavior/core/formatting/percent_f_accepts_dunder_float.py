# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "formatting"
# dimension = "behavior"
# case = "percent_f_accepts_dunder_float"
# subject = "str.percent_format"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""str.percent_format: percent f formatting uses a custom __float__ method returning 2.5"""
# formatting syntax uses only Python built-ins

class FloatOnly:
    def __float__(self):
        return 2.5


obj = FloatOnly()
assert "%.3f" % obj == "2.500"

print("percent_f_accepts_dunder_float OK")
