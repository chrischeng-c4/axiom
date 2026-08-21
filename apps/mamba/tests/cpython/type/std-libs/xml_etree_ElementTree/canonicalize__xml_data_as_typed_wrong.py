# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_etree_ElementTree"
# dimension = "type"
# case = "canonicalize__xml_data_as_typed_wrong"
# subject = "xml.etree.ElementTree.canonicalize(xml_data: typed)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed xml_data"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xml/etree/ElementTree.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed xml_data
# mamba-strict-type: TypeError
"""Type wall: xml.etree.ElementTree.canonicalize(xml_data: typed); call it with the wrong type.

typeshed contract: xml_data is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from xml.etree.ElementTree import canonicalize
try:
    canonicalize(_W())  # xml_data: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
