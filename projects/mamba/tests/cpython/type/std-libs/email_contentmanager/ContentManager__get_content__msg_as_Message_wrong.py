# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email_contentmanager"
# dimension = "type"
# case = "ContentManager__get_content__msg_as_Message_wrong"
# subject = "email.contentmanager.ContentManager.get_content(msg: Message)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed msg"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/email/contentmanager.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed msg
# mamba-strict-type: TypeError
"""Type wall: email.contentmanager.ContentManager.get_content(msg: Message); call it with the wrong type.

typeshed contract: msg is Message. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from email.contentmanager import ContentManager
obj = object.__new__(ContentManager)
try:
    obj.get_content(_W())  # msg: Message <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
