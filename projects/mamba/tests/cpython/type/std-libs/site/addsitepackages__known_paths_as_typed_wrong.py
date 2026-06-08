# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "site"
# dimension = "type"
# case = "addsitepackages__known_paths_as_typed_wrong"
# subject = "site.addsitepackages(known_paths: typed)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed known_paths"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/site.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed known_paths
# mamba-strict-type: TypeError
"""Type wall: site.addsitepackages(known_paths: typed); call it with the wrong type.

typeshed contract: known_paths is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from site import addsitepackages
try:
    addsitepackages(_W())  # known_paths: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
