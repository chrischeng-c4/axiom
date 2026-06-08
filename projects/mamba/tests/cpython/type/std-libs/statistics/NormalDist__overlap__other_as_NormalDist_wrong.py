# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "statistics"
# dimension = "type"
# case = "NormalDist__overlap__other_as_NormalDist_wrong"
# subject = "statistics.NormalDist.overlap(other: NormalDist)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/statistics.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: statistics.NormalDist.overlap(other: NormalDist); call it with the wrong type.

typeshed contract: other is NormalDist. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from statistics import NormalDist
obj = object.__new__(NormalDist)
try:
    obj.overlap(_W())  # other: NormalDist <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
