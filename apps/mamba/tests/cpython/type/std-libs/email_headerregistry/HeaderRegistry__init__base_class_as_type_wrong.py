# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email_headerregistry"
# dimension = "type"
# case = "HeaderRegistry__init__base_class_as_type_wrong"
# subject = "email.headerregistry.HeaderRegistry.__init__(base_class: type)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/email/headerregistry.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: email.headerregistry.HeaderRegistry.__init__(base_class: type); call it with the wrong type.

typeshed contract: base_class is type. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from email.headerregistry import HeaderRegistry
try:
    HeaderRegistry(_W())  # base_class: type <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
