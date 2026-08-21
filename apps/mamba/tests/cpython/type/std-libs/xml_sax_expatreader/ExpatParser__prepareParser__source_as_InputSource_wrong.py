# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_sax_expatreader"
# dimension = "type"
# case = "ExpatParser__prepareParser__source_as_InputSource_wrong"
# subject = "xml.sax.expatreader.ExpatParser.prepareParser(source: InputSource)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xml/sax/expatreader.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xml.sax.expatreader.ExpatParser.prepareParser(source: InputSource); call it with the wrong type.

typeshed contract: source is InputSource. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from xml.sax.expatreader import ExpatParser
obj = object.__new__(ExpatParser)
try:
    obj.prepareParser(_W())  # source: InputSource <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
