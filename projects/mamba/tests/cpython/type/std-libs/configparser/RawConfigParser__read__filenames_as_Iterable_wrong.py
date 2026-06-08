# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "configparser"
# dimension = "type"
# case = "RawConfigParser__read__filenames_as_Iterable_wrong"
# subject = "configparser.RawConfigParser.read(filenames: Iterable)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed filenames"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/configparser.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed filenames
# mamba-strict-type: TypeError
"""Type wall: configparser.RawConfigParser.read(filenames: Iterable); call it with the wrong type.

typeshed contract: filenames is Iterable. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from configparser import RawConfigParser
obj = object.__new__(RawConfigParser)
try:
    obj.read(_W())  # filenames: Iterable <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
