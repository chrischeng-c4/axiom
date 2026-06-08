# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "gettext"
# dimension = "type"
# case = "NullTranslations__add_fallback__fallback_as_NullTranslations_wrong"
# subject = "gettext.NullTranslations.add_fallback(fallback: NullTranslations)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/gettext.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: gettext.NullTranslations.add_fallback(fallback: NullTranslations); call it with the wrong type.

typeshed contract: fallback is NullTranslations. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from gettext import NullTranslations
obj = object.__new__(NullTranslations)
try:
    obj.add_fallback(_W())  # fallback: NullTranslations <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
