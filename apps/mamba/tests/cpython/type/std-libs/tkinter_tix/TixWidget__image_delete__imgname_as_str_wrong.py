# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tkinter_tix"
# dimension = "type"
# case = "TixWidget__image_delete__imgname_as_str_wrong"
# subject = "tkinter.tix.TixWidget.image_delete(imgname: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/tkinter/tix.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: tkinter.tix.TixWidget.image_delete(imgname: str); call it with the wrong type.

typeshed contract: imgname is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from tkinter.tix import TixWidget
obj = object.__new__(TixWidget)
try:
    obj.image_delete(12345)  # imgname: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
