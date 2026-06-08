# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unittest_loader"
# dimension = "type"
# case = "TestLoader__loadTestsFromNames__names_as_Sequence_wrong"
# subject = "unittest.loader.TestLoader.loadTestsFromNames(names: Sequence)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed names"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/unittest/loader.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed names
# mamba-strict-type: TypeError
"""Type wall: unittest.loader.TestLoader.loadTestsFromNames(names: Sequence); call it with the wrong type.

typeshed contract: names is Sequence. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from unittest.loader import TestLoader
obj = object.__new__(TestLoader)
try:
    obj.loadTestsFromNames(_W())  # names: Sequence <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
