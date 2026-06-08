# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "multiprocessing_pool"
# dimension = "type"
# case = "ApplyResult__init__pool_as_Pool_wrong"
# subject = "multiprocessing.pool.ApplyResult.__init__(pool: Pool)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/multiprocessing/pool.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: multiprocessing.pool.ApplyResult.__init__(pool: Pool); call it with the wrong type.

typeshed contract: pool is Pool. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from multiprocessing.pool import ApplyResult
try:
    ApplyResult(_W(), None, None)  # pool: Pool <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
