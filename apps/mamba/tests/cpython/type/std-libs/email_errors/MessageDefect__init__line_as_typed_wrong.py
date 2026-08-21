# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email_errors"
# dimension = "type"
# case = "MessageDefect__init__line_as_typed_wrong"
# subject = "email.errors.MessageDefect.__init__(line: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/email/errors.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: email.errors.MessageDefect.__init__(line: typed); call it with the wrong type.

typeshed contract: line is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from email.errors import MessageDefect
try:
    MessageDefect(_W())  # line: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
