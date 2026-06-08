# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_sax_xmlreader"
# dimension = "type"
# case = "AttributesNSImpl____getitem____name_as__NSName_wrong"
# subject = "xml.sax.xmlreader.AttributesNSImpl.__getitem__(name: _NSName)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xml/sax/xmlreader.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xml.sax.xmlreader.AttributesNSImpl.__getitem__(name: _NSName); call it with the wrong type.

typeshed contract: name is _NSName. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from xml.sax.xmlreader import AttributesNSImpl
obj = object.__new__(AttributesNSImpl)
try:
    obj.__getitem__(_W())  # name: _NSName <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
