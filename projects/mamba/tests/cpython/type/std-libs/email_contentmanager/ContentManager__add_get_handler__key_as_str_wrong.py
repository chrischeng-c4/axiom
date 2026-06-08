# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email_contentmanager"
# dimension = "type"
# case = "ContentManager__add_get_handler__key_as_str_wrong"
# subject = "email.contentmanager.ContentManager.add_get_handler(key: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/email/contentmanager.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: email.contentmanager.ContentManager.add_get_handler(key: str); call it with the wrong type.

typeshed contract: key is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from email.contentmanager import ContentManager
obj = object.__new__(ContentManager)
try:
    obj.add_get_handler(12345, None)  # key: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
