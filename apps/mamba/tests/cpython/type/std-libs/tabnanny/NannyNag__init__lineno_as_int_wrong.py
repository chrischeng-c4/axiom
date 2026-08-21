# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tabnanny"
# dimension = "type"
# case = "NannyNag__init__lineno_as_int_wrong"
# subject = "tabnanny.NannyNag.__init__(lineno: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/tabnanny.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: tabnanny.NannyNag.__init__(lineno: int); call it with the wrong type.

typeshed contract: lineno is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from tabnanny import NannyNag
try:
    NannyNag("not_an_int", "", "")  # lineno: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
