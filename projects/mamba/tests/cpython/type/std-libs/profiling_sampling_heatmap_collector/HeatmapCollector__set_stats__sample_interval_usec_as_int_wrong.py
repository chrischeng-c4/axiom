# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "profiling_sampling_heatmap_collector"
# dimension = "type"
# case = "HeatmapCollector__set_stats__sample_interval_usec_as_int_wrong"
# subject = "profiling.sampling.heatmap_collector.HeatmapCollector.set_stats(sample_interval_usec: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/profiling/sampling/heatmap_collector.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: profiling.sampling.heatmap_collector.HeatmapCollector.set_stats(sample_interval_usec: int); call it with the wrong type.

typeshed contract: sample_interval_usec is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from profiling.sampling.heatmap_collector import HeatmapCollector
obj = object.__new__(HeatmapCollector)
try:
    obj.set_stats("not_an_int", 0.0, 0.0)  # sample_interval_usec: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
