# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "mailbox"
# dimension = "type"
# case = "BabylMessage__set_visible__visible_as__MessageData_wrong"
# subject = "mailbox.BabylMessage.set_visible(visible: _MessageData)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/mailbox.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: mailbox.BabylMessage.set_visible(visible: _MessageData); call it with the wrong type.

typeshed contract: visible is _MessageData. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from mailbox import BabylMessage
obj = object.__new__(BabylMessage)
try:
    obj.set_visible(_W())  # visible: _MessageData <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
