# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "profile"
# dimension = "type"
# case = "Profile__init__timer_as_typed_wrong"
# subject = "profile.Profile.__init__(timer: typed)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed timer"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/profile.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed timer
# mamba-strict-type: TypeError
"""Type wall: profile.Profile.__init__(timer: typed); call it with the wrong type.

typeshed contract: timer is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from profile import Profile
try:
    Profile(_W())  # timer: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
