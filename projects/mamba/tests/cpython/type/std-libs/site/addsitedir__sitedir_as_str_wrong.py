# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "site"
# dimension = "type"
# case = "addsitedir__sitedir_as_str_wrong"
# subject = "site.addsitedir(sitedir: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/site.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: site.addsitedir(sitedir: str); call it with the wrong type.

typeshed contract: sitedir is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from site import addsitedir
try:
    addsitedir(12345)  # sitedir: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
