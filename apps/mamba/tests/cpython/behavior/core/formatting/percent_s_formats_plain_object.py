# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "formatting"
# dimension = "behavior"
# case = "percent_s_formats_plain_object"
# subject = "str.percent_format"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""str.percent_format: percent s formatting renders a plain object with a deterministic repr"""
# formatting syntax uses only Python built-ins

class Marker:
    def __str__(self):
        return "Marker()"


assert "%s" % Marker() == "Marker()"

print("percent_s_formats_plain_object OK")
