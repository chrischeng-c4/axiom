# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "format_map"
# dimension = "behavior"
# case = "format_map_uses_missing_control"
# subject = "str.format_map"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""str.format_map: format_map preserves mapping __missing__ lookup"""
# mapping subclass is defined by the fixture body

class Missing(dict):
    def __missing__(self, key):
        return "<" + key + ">"


assert "{x}".format_map(Missing()) == "<x>"

print("format_map_uses_missing_control OK")
