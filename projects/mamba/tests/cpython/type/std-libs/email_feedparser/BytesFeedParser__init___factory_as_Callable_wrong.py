# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email_feedparser"
# dimension = "type"
# case = "BytesFeedParser__init___factory_as_Callable_wrong"
# subject = "email.feedparser.BytesFeedParser.__init__(_factory: Callable)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/email/feedparser.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: email.feedparser.BytesFeedParser.__init__(_factory: Callable); call it with the wrong type.

typeshed contract: _factory is Callable. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from email.feedparser import BytesFeedParser
try:
    BytesFeedParser(_W())  # _factory: Callable <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
