# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "profiling_sampling_heatmap_collector"
# dimension = "type"
# case = "HeatmapCollector__process_frames__frames_as_Sequence_wrong"
# subject = "profiling.sampling.heatmap_collector.HeatmapCollector.process_frames(frames: Sequence)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed frames"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/profiling/sampling/heatmap_collector.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed frames
# mamba-strict-type: TypeError
"""Type wall: profiling.sampling.heatmap_collector.HeatmapCollector.process_frames(frames: Sequence); call it with the wrong type.

typeshed contract: frames is Sequence. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from profiling.sampling.heatmap_collector import HeatmapCollector
obj = object.__new__(HeatmapCollector)
try:
    obj.process_frames(_W(), 0)  # frames: Sequence <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
