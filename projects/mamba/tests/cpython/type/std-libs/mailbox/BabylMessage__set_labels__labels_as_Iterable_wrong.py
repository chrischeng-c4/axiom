# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "mailbox"
# dimension = "type"
# case = "BabylMessage__set_labels__labels_as_Iterable_wrong"
# subject = "mailbox.BabylMessage.set_labels(labels: Iterable)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/mailbox.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: mailbox.BabylMessage.set_labels(labels: Iterable); call it with the wrong type.

typeshed contract: labels is Iterable. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from mailbox import BabylMessage
obj = object.__new__(BabylMessage)
try:
    obj.set_labels(_W())  # labels: Iterable <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
