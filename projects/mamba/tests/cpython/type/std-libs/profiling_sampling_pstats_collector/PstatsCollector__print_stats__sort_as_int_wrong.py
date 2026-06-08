# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "profiling_sampling_pstats_collector"
# dimension = "type"
# case = "PstatsCollector__print_stats__sort_as_int_wrong"
# subject = "profiling.sampling.pstats_collector.PstatsCollector.print_stats(sort: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/profiling/sampling/pstats_collector.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: profiling.sampling.pstats_collector.PstatsCollector.print_stats(sort: int); call it with the wrong type.

typeshed contract: sort is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from profiling.sampling.pstats_collector import PstatsCollector
obj = object.__new__(PstatsCollector)
try:
    obj.print_stats("not_an_int")  # sort: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
