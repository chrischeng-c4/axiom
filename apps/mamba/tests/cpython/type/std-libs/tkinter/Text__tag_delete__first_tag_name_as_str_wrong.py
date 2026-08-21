# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tkinter"
# dimension = "type"
# case = "Text__tag_delete__first_tag_name_as_str_wrong"
# subject = "tkinter.Text.tag_delete(first_tag_name: str)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed first_tag_name"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/tkinter.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed first_tag_name
# mamba-strict-type: TypeError
"""Type wall: tkinter.Text.tag_delete(first_tag_name: str); call it with the wrong type.

typeshed contract: first_tag_name is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from tkinter import Text
obj = object.__new__(Text)
try:
    obj.tag_delete(12345)  # first_tag_name: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
