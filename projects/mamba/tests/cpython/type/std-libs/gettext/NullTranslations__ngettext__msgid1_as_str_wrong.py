# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "gettext"
# dimension = "type"
# case = "NullTranslations__ngettext__msgid1_as_str_wrong"
# subject = "gettext.NullTranslations.ngettext(msgid1: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/gettext.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: gettext.NullTranslations.ngettext(msgid1: str); call it with the wrong type.

typeshed contract: msgid1 is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from gettext import NullTranslations
obj = object.__new__(NullTranslations)
try:
    obj.ngettext(12345, "", 0)  # msgid1: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
