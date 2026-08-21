# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "difflib"
# dimension = "type"
# case = "HtmlDiff__make_file__fromlines_as_Sequence_wrong"
# subject = "difflib.HtmlDiff.make_file(fromlines: Sequence)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed fromlines"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/difflib.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed fromlines
# mamba-strict-type: TypeError
"""Type wall: difflib.HtmlDiff.make_file(fromlines: Sequence); call it with the wrong type.

typeshed contract: fromlines is Sequence. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from difflib import HtmlDiff
obj = object.__new__(HtmlDiff)
try:
    obj.make_file(_W(), None)  # fromlines: Sequence <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
