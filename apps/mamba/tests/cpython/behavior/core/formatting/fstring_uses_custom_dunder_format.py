# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "formatting"
# dimension = "behavior"
# case = "fstring_uses_custom_dunder_format"
# subject = "fstring.float_format"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""fstring.float_format: f-string formatting dispatches a custom __format__ method with the .3f spec"""
# formatting syntax uses only Python built-ins

class Marker:
    def __format__(self, spec):
        assert spec == ".3f"
        return f"formatted({spec})"


obj = Marker()
assert f"{obj:.3f}" == "formatted(.3f)"

print("fstring_uses_custom_dunder_format OK")
