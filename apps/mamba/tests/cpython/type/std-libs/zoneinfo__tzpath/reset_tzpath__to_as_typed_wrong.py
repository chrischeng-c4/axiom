# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zoneinfo__tzpath"
# dimension = "type"
# case = "reset_tzpath__to_as_typed_wrong"
# subject = "zoneinfo._tzpath.reset_tzpath(to: typed)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed to"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/zoneinfo/_tzpath.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed to
# mamba-strict-type: TypeError
"""Type wall: zoneinfo._tzpath.reset_tzpath(to: typed); call it with the wrong type.

typeshed contract: to is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from zoneinfo._tzpath import reset_tzpath
try:
    reset_tzpath(_W())  # to: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
