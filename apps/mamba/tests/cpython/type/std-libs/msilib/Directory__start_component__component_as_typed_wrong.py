# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "msilib"
# dimension = "type"
# case = "Directory__start_component__component_as_typed_wrong"
# subject = "msilib.Directory.start_component(component: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/msilib.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: msilib.Directory.start_component(component: typed); call it with the wrong type.

typeshed contract: component is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from msilib import Directory
obj = object.__new__(Directory)
try:
    obj.start_component(_W())  # component: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
