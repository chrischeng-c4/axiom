# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_request"
# dimension = "type"
# case = "HTTPSHandler__init__debuglevel_as_int_wrong"
# subject = "urllib.request.HTTPSHandler.__init__(debuglevel: int)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed debuglevel"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/urllib/request.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed debuglevel
# mamba-strict-type: TypeError
"""Type wall: urllib.request.HTTPSHandler.__init__(debuglevel: int); call it with the wrong type.

typeshed contract: debuglevel is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from urllib.request import HTTPSHandler
try:
    HTTPSHandler("not_an_int")  # debuglevel: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
