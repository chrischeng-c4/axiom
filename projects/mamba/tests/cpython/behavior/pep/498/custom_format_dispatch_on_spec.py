# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "498"
# dimension = "behavior"
# case = "custom_format_dispatch_on_spec"
# subject = "fstring.format_spec"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""fstring.format_spec: __format__ receives the raw spec: a Marker whose __format__ returns '*' when spec is empty else the spec gives f'{m}' '*', f'{m:}' '*', f'{m:x}' 'x'"""
# the format spec string is passed verbatim to __format__

class Marker:
    def __format__(self, spec):
        return "*" if not spec else spec

m = Marker()
assert f"{m}" == "*"
assert f"{m:}" == "*"
assert f"{m:x}" == "x"

print("custom_format_dispatch_on_spec OK")
