# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pydoc"
# dimension = "type"
# case = "HTMLDoc__modpkglink__modpkginfo_as_tuple_wrong"
# subject = "pydoc.HTMLDoc.modpkglink(modpkginfo: tuple)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed modpkginfo"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/pydoc.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed modpkginfo
# mamba-strict-type: TypeError
"""Type wall: pydoc.HTMLDoc.modpkglink(modpkginfo: tuple); call it with the wrong type.

typeshed contract: modpkginfo is tuple. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from pydoc import HTMLDoc
obj = object.__new__(HTMLDoc)
try:
    obj.modpkglink(12345)  # modpkginfo: tuple <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
