# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "importlib_resources_readers"
# dimension = "type"
# case = "remove_duplicates__items_as_Iterable_wrong"
# subject = "importlib.resources.readers.remove_duplicates(items: Iterable)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/importlib/resources/readers.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: importlib.resources.readers.remove_duplicates(items: Iterable); call it with the wrong type.

typeshed contract: items is Iterable. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from importlib.resources.readers import remove_duplicates
try:
    remove_duplicates(_W())  # items: Iterable <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
