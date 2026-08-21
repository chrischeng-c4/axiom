# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "lib2to3_pgen2_driver"
# dimension = "type"
# case = "Driver__init__grammar_as_Grammar_wrong"
# subject = "lib2to3.pgen2.driver.Driver.__init__(grammar: Grammar)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/lib2to3/pgen2/driver.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: lib2to3.pgen2.driver.Driver.__init__(grammar: Grammar); call it with the wrong type.

typeshed contract: grammar is Grammar. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from lib2to3.pgen2.driver import Driver
try:
    Driver(_W())  # grammar: Grammar <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
