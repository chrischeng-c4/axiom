# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "formatting"
# dimension = "behavior"
# case = "str_format_uses_custom_dunder_format"
# subject = "str.format"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""str.format: str.format dispatches a custom __format__ method with the .3f spec"""
# formatting syntax uses only Python built-ins

class Marker:
    def __format__(self, spec):
        assert spec == ".3f"
        return f"formatted({spec})"


obj = Marker()
assert "{:.3f}".format(obj) == "formatted(.3f)"

print("str_format_uses_custom_dunder_format OK")
