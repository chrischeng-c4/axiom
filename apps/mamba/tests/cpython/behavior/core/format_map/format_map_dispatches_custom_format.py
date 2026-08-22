# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "format_map"
# dimension = "behavior"
# case = "format_map_dispatches_custom_format"
# subject = "str.format_map"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""str.format_map: format_map invokes class-level __format__ exactly once with .3f and preserves its sentinel"""
# custom class is defined by the fixture body

class Marker:
    def __init__(self):
        self.calls = 0
        self.last_spec = ""

    def __format__(self, spec):
        self.calls += 1
        self.last_spec = spec
        return f"format_map_sentinel:{spec}"


marker = Marker()
formatted = "{x:.3f}".format_map({"x": marker})
assert (formatted, marker.calls, marker.last_spec) == (
    "format_map_sentinel:.3f",
    1,
    ".3f",
)

print("format_map_dispatches_custom_format OK")
