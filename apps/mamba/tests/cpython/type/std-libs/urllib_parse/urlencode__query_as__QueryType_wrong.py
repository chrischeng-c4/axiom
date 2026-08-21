# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_parse"
# dimension = "type"
# case = "urlencode__query_as__QueryType_wrong"
# subject = "urllib.parse.urlencode(query: _QueryType)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/urllib/parse.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: urllib.parse.urlencode(query: _QueryType); call it with the wrong type.

typeshed contract: query is _QueryType. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from urllib.parse import urlencode
try:
    urlencode(_W())  # query: _QueryType <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
