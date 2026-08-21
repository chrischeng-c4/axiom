# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "distutils_version"
# dimension = "type"
# case = "Version__init__vstring_as_typed_wrong"
# subject = "distutils.version.Version.__init__(vstring: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/distutils/version.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: distutils.version.Version.__init__(vstring: typed); call it with the wrong type.

typeshed contract: vstring is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from distutils.version import Version
try:
    Version(_W())  # vstring: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
