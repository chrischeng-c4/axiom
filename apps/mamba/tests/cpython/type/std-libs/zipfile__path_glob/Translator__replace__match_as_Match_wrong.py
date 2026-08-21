# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zipfile__path_glob"
# dimension = "type"
# case = "Translator__replace__match_as_Match_wrong"
# subject = "zipfile._path.glob.Translator.replace(match: Match)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed match"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/zipfile/_path/glob.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed match
# mamba-strict-type: TypeError
"""Type wall: zipfile._path.glob.Translator.replace(match: Match); call it with the wrong type.

typeshed contract: match is Match. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from zipfile._path.glob import Translator
obj = object.__new__(Translator)
try:
    obj.replace(_W())  # match: Match <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
