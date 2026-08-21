# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_sax__exceptions"
# dimension = "type"
# case = "SAXParseException__init__msg_as_str_wrong"
# subject = "xml.sax._exceptions.SAXParseException.__init__(msg: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xml/sax/_exceptions.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xml.sax._exceptions.SAXParseException.__init__(msg: str); call it with the wrong type.

typeshed contract: msg is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from xml.sax._exceptions import SAXParseException
try:
    SAXParseException(12345, None, None)  # msg: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
