# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "profiling_sampling_collector"
# dimension = "type"
# case = "normalize_location__location_as__Location_wrong"
# subject = "profiling.sampling.collector.normalize_location(location: _Location)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/profiling/sampling/collector.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: profiling.sampling.collector.normalize_location(location: _Location); call it with the wrong type.

typeshed contract: location is _Location. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from profiling.sampling.collector import normalize_location
try:
    normalize_location(_W())  # location: _Location <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
