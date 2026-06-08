# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "profiling_sampling_jsonl_collector"
# dimension = "type"
# case = "JsonlCollector__collect__stack_frames_as_typed_wrong"
# subject = "profiling.sampling.jsonl_collector.JsonlCollector.collect(stack_frames: typed)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed stack_frames"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/profiling/sampling/jsonl_collector.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed stack_frames
# mamba-strict-type: TypeError
"""Type wall: profiling.sampling.jsonl_collector.JsonlCollector.collect(stack_frames: typed); call it with the wrong type.

typeshed contract: stack_frames is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from profiling.sampling.jsonl_collector import JsonlCollector
obj = object.__new__(JsonlCollector)
try:
    obj.collect(_W())  # stack_frames: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
