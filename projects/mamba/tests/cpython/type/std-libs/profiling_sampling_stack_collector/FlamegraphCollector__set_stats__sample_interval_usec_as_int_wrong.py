# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "profiling_sampling_stack_collector"
# dimension = "type"
# case = "FlamegraphCollector__set_stats__sample_interval_usec_as_int_wrong"
# subject = "profiling.sampling.stack_collector.FlamegraphCollector.set_stats(sample_interval_usec: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/profiling/sampling/stack_collector.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: profiling.sampling.stack_collector.FlamegraphCollector.set_stats(sample_interval_usec: int); call it with the wrong type.

typeshed contract: sample_interval_usec is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from profiling.sampling.stack_collector import FlamegraphCollector
obj = object.__new__(FlamegraphCollector)
try:
    obj.set_stats("not_an_int", 0.0, 0.0)  # sample_interval_usec: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
