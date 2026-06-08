# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "profiling_sampling_jsonl_collector"
# dimension = "type"
# case = "JsonlCollector__process_frames__frames_as_Sequence_wrong"
# subject = "profiling.sampling.jsonl_collector.JsonlCollector.process_frames(frames: Sequence)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed frames"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/profiling/sampling/jsonl_collector.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed frames
# mamba-strict-type: TypeError
"""Type wall: profiling.sampling.jsonl_collector.JsonlCollector.process_frames(frames: Sequence); call it with the wrong type.

typeshed contract: frames is Sequence. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from profiling.sampling.jsonl_collector import JsonlCollector
obj = object.__new__(JsonlCollector)
try:
    obj.process_frames(_W(), 0)  # frames: Sequence <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
