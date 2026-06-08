# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "profiling_sampling_collector"
# dimension = "type"
# case = "iter_async_frames__awaited_info_list_as_Sequence_wrong"
# subject = "profiling.sampling.collector.iter_async_frames(awaited_info_list: Sequence)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed awaited_info_list"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/profiling/sampling/collector.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed awaited_info_list
# mamba-strict-type: TypeError
"""Type wall: profiling.sampling.collector.iter_async_frames(awaited_info_list: Sequence); call it with the wrong type.

typeshed contract: awaited_info_list is Sequence. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from profiling.sampling.collector import iter_async_frames
try:
    iter_async_frames(_W())  # awaited_info_list: Sequence <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
