# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "profiling_sampling_heatmap_collector"
# dimension = "type"
# case = "HeatmapCollector__export__output_path_as_StrOrBytesPath_wrong"
# subject = "profiling.sampling.heatmap_collector.HeatmapCollector.export(output_path: StrOrBytesPath)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/profiling/sampling/heatmap_collector.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: profiling.sampling.heatmap_collector.HeatmapCollector.export(output_path: StrOrBytesPath); call it with the wrong type.

typeshed contract: output_path is StrOrBytesPath. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from profiling.sampling.heatmap_collector import HeatmapCollector
obj = object.__new__(HeatmapCollector)
try:
    obj.export(_W())  # output_path: StrOrBytesPath <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
