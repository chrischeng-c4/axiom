# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "codecs"
# dimension = "type"
# case = "StreamReaderWriter__readline__size_as_typed_wrong"
# subject = "codecs.StreamReaderWriter.readline(size: typed)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/codecs.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: codecs.StreamReaderWriter.readline(size: typed); call it with the wrong type.

typeshed contract: size is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from codecs import StreamReaderWriter
obj = object.__new__(StreamReaderWriter)
try:
    obj.readline(_W())  # size: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
