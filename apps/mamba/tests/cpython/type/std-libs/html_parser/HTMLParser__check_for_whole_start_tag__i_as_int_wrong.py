# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "html_parser"
# dimension = "type"
# case = "HTMLParser__check_for_whole_start_tag__i_as_int_wrong"
# subject = "html.parser.HTMLParser.check_for_whole_start_tag(i: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/html/parser.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: html.parser.HTMLParser.check_for_whole_start_tag(i: int); call it with the wrong type.

typeshed contract: i is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from html.parser import HTMLParser
obj = object.__new__(HTMLParser)
try:
    obj.check_for_whole_start_tag("not_an_int")  # i: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
