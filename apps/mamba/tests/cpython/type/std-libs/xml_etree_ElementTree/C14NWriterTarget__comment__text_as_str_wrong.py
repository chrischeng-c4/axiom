# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_etree_ElementTree"
# dimension = "type"
# case = "C14NWriterTarget__comment__text_as_str_wrong"
# subject = "xml.etree.ElementTree.C14NWriterTarget.comment(text: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xml/etree/ElementTree.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xml.etree.ElementTree.C14NWriterTarget.comment(text: str); call it with the wrong type.

typeshed contract: text is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from xml.etree.ElementTree import C14NWriterTarget
obj = object.__new__(C14NWriterTarget)
try:
    obj.comment(12345)  # text: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
