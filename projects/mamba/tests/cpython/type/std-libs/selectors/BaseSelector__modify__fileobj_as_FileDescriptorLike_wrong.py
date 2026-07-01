# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "selectors"
# dimension = "type"
# case = "BaseSelector__modify__fileobj_as_FileDescriptorLike_wrong"
# subject = "selectors.BaseSelector.modify(fileobj: FileDescriptorLike)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/selectors.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: selectors.BaseSelector.modify(fileobj: FileDescriptorLike); call it with the wrong type.

typeshed contract: fileobj is FileDescriptorLike. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from selectors import BaseSelector
obj = object.__new__(BaseSelector)
try:
    obj.modify(_W(), 0)  # fileobj: FileDescriptorLike <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
