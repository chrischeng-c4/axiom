# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "lib2to3_pgen2_driver"
# dimension = "type"
# case = "Driver__parse_tokens__tokens_as_Iterable_wrong"
# subject = "lib2to3.pgen2.driver.Driver.parse_tokens(tokens: Iterable)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/lib2to3/pgen2/driver.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: lib2to3.pgen2.driver.Driver.parse_tokens(tokens: Iterable); call it with the wrong type.

typeshed contract: tokens is Iterable. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from lib2to3.pgen2.driver import Driver
obj = object.__new__(Driver)
try:
    obj.parse_tokens(_W())  # tokens: Iterable <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
