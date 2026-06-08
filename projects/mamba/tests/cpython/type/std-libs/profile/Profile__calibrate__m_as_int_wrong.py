# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "profile"
# dimension = "type"
# case = "Profile__calibrate__m_as_int_wrong"
# subject = "profile.Profile.calibrate(m: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/profile.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: profile.Profile.calibrate(m: int); call it with the wrong type.

typeshed contract: m is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from profile import Profile
obj = object.__new__(Profile)
try:
    obj.calibrate("not_an_int")  # m: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
