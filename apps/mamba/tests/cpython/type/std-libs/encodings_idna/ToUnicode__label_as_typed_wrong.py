# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "encodings_idna"
# dimension = "type"
# case = "ToUnicode__label_as_typed_wrong"
# subject = "encodings.idna.ToUnicode(label: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/encodings/idna.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: encodings.idna.ToUnicode(label: typed); call it with the wrong type.

typeshed contract: label is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from encodings.idna import ToUnicode
try:
    ToUnicode(_W())  # label: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
