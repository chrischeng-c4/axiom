# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_zstd"
# dimension = "type"
# case = "ZstdDecompressor____new____zstd_dict_as_typed_wrong"
# subject = "_zstd.ZstdDecompressor.__new__(zstd_dict: typed)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed zstd_dict"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_zstd.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed zstd_dict
# mamba-strict-type: TypeError
"""Type wall: _zstd.ZstdDecompressor.__new__(zstd_dict: typed); call it with the wrong type.

typeshed contract: zstd_dict is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from _zstd import ZstdDecompressor
obj = object.__new__(ZstdDecompressor)
try:
    obj.__new__(_W())  # zstd_dict: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
