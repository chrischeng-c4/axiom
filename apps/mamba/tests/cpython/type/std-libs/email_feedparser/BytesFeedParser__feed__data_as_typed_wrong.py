# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email_feedparser"
# dimension = "type"
# case = "BytesFeedParser__feed__data_as_typed_wrong"
# subject = "email.feedparser.BytesFeedParser.feed(data: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/email/feedparser.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: email.feedparser.BytesFeedParser.feed(data: typed); call it with the wrong type.

typeshed contract: data is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from email.feedparser import BytesFeedParser
obj = object.__new__(BytesFeedParser)
try:
    obj.feed(_W())  # data: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
