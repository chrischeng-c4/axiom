# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "html_parser"
# dimension = "type"
# case = "HTMLParser__goahead__end_as_bool_wrong"
# subject = "html.parser.HTMLParser.goahead(end: bool)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed end"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/html/parser.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed end
# mamba-strict-type: TypeError
"""Type wall: html.parser.HTMLParser.goahead(end: bool); call it with the wrong type.

typeshed contract: end is bool. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from html.parser import HTMLParser
obj = object.__new__(HTMLParser)
try:
    obj.goahead("not_a_bool")  # end: bool <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
