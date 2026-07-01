# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "multiprocessing_pool"
# dimension = "type"
# case = "Pool__Process__ctx_as_DefaultContext_wrong"
# subject = "multiprocessing.pool.Pool.Process(ctx: DefaultContext)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/multiprocessing/pool.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: multiprocessing.pool.Pool.Process(ctx: DefaultContext); call it with the wrong type.

typeshed contract: ctx is DefaultContext. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from multiprocessing.pool import Pool
try:
    Pool.Process(_W())  # ctx: DefaultContext <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
