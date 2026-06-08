# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "distutils_dist"
# dimension = "type"
# case = "Distribution__parse_config_files__filenames_as_typed_wrong"
# subject = "distutils.dist.Distribution.parse_config_files(filenames: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/distutils/dist.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: distutils.dist.Distribution.parse_config_files(filenames: typed); call it with the wrong type.

typeshed contract: filenames is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from distutils.dist import Distribution
obj = object.__new__(Distribution)
try:
    obj.parse_config_files(_W())  # filenames: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
