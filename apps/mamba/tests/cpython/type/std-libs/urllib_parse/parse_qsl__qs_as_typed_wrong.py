# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_parse"
# dimension = "type"
# case = "parse_qsl__qs_as_typed_wrong"
# subject = "urllib.parse.parse_qsl(qs: typed)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed qs"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/urllib/parse.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed qs
# mamba-strict-type: TypeError
"""Type wall: urllib.parse.parse_qsl(qs: typed); call it with the wrong type.

typeshed contract: qs is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from urllib.parse import parse_qsl
try:
    parse_qsl(_W())  # qs: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
