# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_sax_expatreader"
# dimension = "type"
# case = "ExpatLocator__init__parser_as_ExpatParser_wrong"
# subject = "xml.sax.expatreader.ExpatLocator.__init__(parser: ExpatParser)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xml/sax/expatreader.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xml.sax.expatreader.ExpatLocator.__init__(parser: ExpatParser); call it with the wrong type.

typeshed contract: parser is ExpatParser. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from xml.sax.expatreader import ExpatLocator
try:
    ExpatLocator(_W())  # parser: ExpatParser <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
