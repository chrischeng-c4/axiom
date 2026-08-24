# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "formatting"
# dimension = "errors"
# case = "fstring_str_subclass_keeps_value_error"
# subject = "fstring.float_format"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""fstring.float_format: f-string .3f on a str subclass preserves ValueError taxonomy"""
class StrSub(str):
    pass


try:
    f"{StrSub('abc'):.3f}"
except ValueError as exc:
    assert str(exc) == "Unknown format code 'f' for object of type 'StrSub'"
else:
    raise AssertionError("expected ValueError")

print("fstring_str_subclass_keeps_value_error OK")
