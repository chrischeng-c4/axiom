# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_etree_ElementTree"
# dimension = "type"
# case = "ElementTree__write_c14n__file_as__FileWriteC14N_wrong"
# subject = "xml.etree.ElementTree.ElementTree.write_c14n(file: _FileWriteC14N)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xml/etree/ElementTree.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xml.etree.ElementTree.ElementTree.write_c14n(file: _FileWriteC14N); call it with the wrong type.

typeshed contract: file is _FileWriteC14N. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from xml.etree.ElementTree import ElementTree
obj = object.__new__(ElementTree)
try:
    obj.write_c14n(_W())  # file: _FileWriteC14N <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
