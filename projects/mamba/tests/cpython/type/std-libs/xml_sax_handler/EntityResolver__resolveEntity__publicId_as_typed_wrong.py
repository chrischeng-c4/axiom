# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_sax_handler"
# dimension = "type"
# case = "EntityResolver__resolveEntity__publicId_as_typed_wrong"
# subject = "xml.sax.handler.EntityResolver.resolveEntity(publicId: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xml/sax/handler.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xml.sax.handler.EntityResolver.resolveEntity(publicId: typed); call it with the wrong type.

typeshed contract: publicId is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from xml.sax.handler import EntityResolver
obj = object.__new__(EntityResolver)
try:
    obj.resolveEntity(_W(), "")  # publicId: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
