# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "select"
# dimension = "type"
# case = "poll__register__fd_as_FileDescriptorLike_wrong"
# subject = "select.poll.register(fd: FileDescriptorLike)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/select.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: select.poll.register(fd: FileDescriptorLike); call it with the wrong type.

typeshed contract: fd is FileDescriptorLike. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from select import poll
obj = object.__new__(poll)
try:
    obj.register(_W())  # fd: FileDescriptorLike <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
