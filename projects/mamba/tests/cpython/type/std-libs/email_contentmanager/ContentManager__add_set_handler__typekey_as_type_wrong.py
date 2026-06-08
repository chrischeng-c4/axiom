# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email_contentmanager"
# dimension = "type"
# case = "ContentManager__add_set_handler__typekey_as_type_wrong"
# subject = "email.contentmanager.ContentManager.add_set_handler(typekey: type)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed typekey"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/email/contentmanager.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed typekey
# mamba-strict-type: TypeError
"""Type wall: email.contentmanager.ContentManager.add_set_handler(typekey: type); call it with the wrong type.

typeshed contract: typekey is type. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from email.contentmanager import ContentManager
obj = object.__new__(ContentManager)
try:
    obj.add_set_handler(_W(), None)  # typekey: type <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
