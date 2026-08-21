# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "configparser"
# dimension = "type"
# case = "RawConfigParser__set__section_as__SectionName_wrong"
# subject = "configparser.RawConfigParser.set(section: _SectionName)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/configparser.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: configparser.RawConfigParser.set(section: _SectionName); call it with the wrong type.

typeshed contract: section is _SectionName. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from configparser import RawConfigParser
obj = object.__new__(RawConfigParser)
try:
    obj.set(_W(), "")  # section: _SectionName <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
