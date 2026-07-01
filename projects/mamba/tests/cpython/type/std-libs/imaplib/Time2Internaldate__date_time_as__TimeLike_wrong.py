# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "imaplib"
# dimension = "type"
# case = "Time2Internaldate__date_time_as__TimeLike_wrong"
# subject = "imaplib.Time2Internaldate(date_time: _TimeLike)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/imaplib.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: imaplib.Time2Internaldate(date_time: _TimeLike); call it with the wrong type.

typeshed contract: date_time is _TimeLike. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from imaplib import Time2Internaldate
try:
    Time2Internaldate(_W())  # date_time: _TimeLike <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
