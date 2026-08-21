# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "compression_zstd"
# dimension = "type"
# case = "train_dict__samples_as_Iterable_wrong"
# subject = "compression.zstd.train_dict(samples: Iterable)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/compression/zstd.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: compression.zstd.train_dict(samples: Iterable); call it with the wrong type.

typeshed contract: samples is Iterable. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from compression.zstd import train_dict
try:
    train_dict(_W(), 0)  # samples: Iterable <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
