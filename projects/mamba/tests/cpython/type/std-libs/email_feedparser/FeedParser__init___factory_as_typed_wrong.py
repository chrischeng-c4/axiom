# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email_feedparser"
# dimension = "type"
# case = "FeedParser__init___factory_as_typed_wrong"
# subject = "email.feedparser.FeedParser.__init__(_factory: typed)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed _factory"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/email/feedparser.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed _factory
# mamba-strict-type: TypeError
"""Type wall: email.feedparser.FeedParser.__init__(_factory: typed); call it with the wrong type.

typeshed contract: _factory is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from email.feedparser import FeedParser
try:
    FeedParser(_W())  # _factory: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
