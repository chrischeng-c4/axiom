# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "distutils_dist"
# dimension = "type"
# case = "DistributionMetadata__write_pkg_file__file_as_SupportsWrite_wrong"
# subject = "distutils.dist.DistributionMetadata.write_pkg_file(file: SupportsWrite)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/distutils/dist.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: distutils.dist.DistributionMetadata.write_pkg_file(file: SupportsWrite); call it with the wrong type.

typeshed contract: file is SupportsWrite. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from distutils.dist import DistributionMetadata
obj = object.__new__(DistributionMetadata)
try:
    obj.write_pkg_file(_W())  # file: SupportsWrite <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
