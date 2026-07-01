# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cProfile"
# dimension = "type"
# case = "Profile__dump_stats__file_as_StrOrBytesPath_wrong"
# subject = "cProfile.Profile.dump_stats(file: StrOrBytesPath)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/cProfile.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: cProfile.Profile.dump_stats(file: StrOrBytesPath); call it with the wrong type.

typeshed contract: file is StrOrBytesPath. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from cProfile import Profile
obj = object.__new__(Profile)
try:
    obj.dump_stats(_W())  # file: StrOrBytesPath <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
