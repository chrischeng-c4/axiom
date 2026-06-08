# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_sax_saxutils"
# dimension = "type"
# case = "XMLGenerator__ignorableWhitespace__content_as_str_wrong"
# subject = "xml.sax.saxutils.XMLGenerator.ignorableWhitespace(content: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xml/sax/saxutils.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xml.sax.saxutils.XMLGenerator.ignorableWhitespace(content: str); call it with the wrong type.

typeshed contract: content is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from xml.sax.saxutils import XMLGenerator
obj = object.__new__(XMLGenerator)
try:
    obj.ignorableWhitespace(12345)  # content: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
