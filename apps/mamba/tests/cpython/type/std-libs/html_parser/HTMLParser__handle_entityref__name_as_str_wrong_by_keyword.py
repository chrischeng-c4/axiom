# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "html_parser"
# dimension = "type"
# case = "HTMLParser__handle_entityref__name_as_str_wrong_by_keyword"
# subject = "html.parser.HTMLParser.handle_entityref(name: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/html/parser.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: html.parser.HTMLParser.handle_entityref(name: str); call it with
the wrong type, BY KEYWORD.

Same wrong-typed-arg contract as HTMLParser__handle_entityref__name_as_str_wrong
(the positional twin), but `name` is passed as `name=12345` instead of
positionally. The ① type-wall enforcement hook must align a
`CallArg::Keyword{name, value}` to its like-named `ParamSig` and run the same
scalar check positional args get, not stop enforcement at the first
non-positional argument (#881).

typeshed contract: name is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from html.parser import HTMLParser
obj = object.__new__(HTMLParser)
try:
    obj.handle_entityref(name=12345)  # name: str <- wrong-typed, by keyword
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
