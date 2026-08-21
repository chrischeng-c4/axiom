# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_sax_xmlreader"
# dimension = "type"
# case = "AttributesNSImpl__getNameByQName__name_as_str_wrong"
# subject = "xml.sax.xmlreader.AttributesNSImpl.getNameByQName(name: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xml/sax/xmlreader.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xml.sax.xmlreader.AttributesNSImpl.getNameByQName(name: str); call it with the wrong type.

typeshed contract: name is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from xml.sax.xmlreader import AttributesNSImpl
obj = object.__new__(AttributesNSImpl)
try:
    obj.getNameByQName(12345)  # name: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
