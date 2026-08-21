# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "select"
# dimension = "type"
# case = "kevent__init__ident_as_FileDescriptorLike_wrong"
# subject = "select.kevent.__init__(ident: FileDescriptorLike)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/select.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: select.kevent.__init__(ident: FileDescriptorLike); call it with the wrong type.

typeshed contract: ident is FileDescriptorLike. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from select import kevent
try:
    kevent(_W())  # ident: FileDescriptorLike <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
