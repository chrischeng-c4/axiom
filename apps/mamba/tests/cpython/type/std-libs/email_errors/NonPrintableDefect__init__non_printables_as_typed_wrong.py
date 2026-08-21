# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email_errors"
# dimension = "type"
# case = "NonPrintableDefect__init__non_printables_as_typed_wrong"
# subject = "email.errors.NonPrintableDefect.__init__(non_printables: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/email/errors.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: email.errors.NonPrintableDefect.__init__(non_printables: typed); call it with the wrong type.

typeshed contract: non_printables is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from email.errors import NonPrintableDefect
try:
    NonPrintableDefect(_W())  # non_printables: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
