# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_response"
# dimension = "type"
# case = "addbase__writelines__lines_as_Iterable_wrong"
# subject = "urllib.response.addbase.writelines(lines: Iterable)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/urllib/response.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: urllib.response.addbase.writelines(lines: Iterable); call it with the wrong type.

typeshed contract: lines is Iterable. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from urllib.response import addbase
obj = object.__new__(addbase)
try:
    obj.writelines(_W())  # lines: Iterable <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
