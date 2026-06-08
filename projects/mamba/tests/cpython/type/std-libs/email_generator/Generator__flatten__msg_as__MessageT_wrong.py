# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email_generator"
# dimension = "type"
# case = "Generator__flatten__msg_as__MessageT_wrong"
# subject = "email.generator.Generator.flatten(msg: _MessageT)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed msg"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/email/generator.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed msg
# mamba-strict-type: TypeError
"""Type wall: email.generator.Generator.flatten(msg: _MessageT); call it with the wrong type.

typeshed contract: msg is _MessageT. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from email.generator import Generator
obj = object.__new__(Generator)
try:
    obj.flatten(_W())  # msg: _MessageT <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
