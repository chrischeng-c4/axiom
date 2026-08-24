# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "formatting"
# dimension = "errors"
# case = "str_format_str_subclass_keeps_value_error"
# subject = "str.format"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""str.format: str.format .3f on a str subclass preserves ValueError taxonomy"""
class StrSub(str):
    pass


try:
    "{:.3f}".format(StrSub("abc"))
except ValueError as exc:
    assert str(exc) == "Unknown format code 'f' for object of type 'StrSub'"
else:
    raise AssertionError("expected ValueError")

print("str_format_str_subclass_keeps_value_error OK")
