# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_request"
# dimension = "type"
# case = "urlopen__url_as_typed_wrong"
# subject = "urllib.request.urlopen(url: typed)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed url"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/urllib/request.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed url
# mamba-strict-type: TypeError
"""Type wall: urllib.request.urlopen(url: typed); call it with the wrong type.

typeshed contract: url is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from urllib.request import urlopen
try:
    urlopen(_W())  # url: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
