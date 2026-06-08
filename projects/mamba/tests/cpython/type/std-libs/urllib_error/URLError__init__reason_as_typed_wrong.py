# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_error"
# dimension = "type"
# case = "URLError__init__reason_as_typed_wrong"
# subject = "urllib.error.URLError.__init__(reason: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/urllib/error.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: urllib.error.URLError.__init__(reason: typed); call it with the wrong type.

typeshed contract: reason is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from urllib.error import URLError
try:
    URLError(_W())  # reason: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
