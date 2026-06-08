# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "profile"
# dimension = "type"
# case = "Profile__runctx__cmd_as_str_wrong"
# subject = "profile.Profile.runctx(cmd: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/profile.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: profile.Profile.runctx(cmd: str); call it with the wrong type.

typeshed contract: cmd is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from profile import Profile
obj = object.__new__(Profile)
try:
    obj.runctx(12345, None, None)  # cmd: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
