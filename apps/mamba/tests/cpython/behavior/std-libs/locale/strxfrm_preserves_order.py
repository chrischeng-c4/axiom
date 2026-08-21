# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "locale"
# dimension = "behavior"
# case = "strxfrm_preserves_order"
# subject = "locale.strxfrm"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_locale.py"
# status = "filled"
# ///
"""locale.strxfrm: strxfrm maps a string to a str sort key whose comparison preserves the original order"""
import locale

# strxfrm maps a string to a sort key; transformed keys preserve order.
assert locale.strxfrm("a") < locale.strxfrm("b"), "strxfrm a<b"
assert isinstance(locale.strxfrm("a"), str), "strxfrm -> str"

print("strxfrm_preserves_order OK")
