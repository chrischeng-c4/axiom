# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "csv"
# dimension = "type"
# case = "DictReader__init__f_as_Iterable_wrong"
# subject = "csv.DictReader.__init__(f: Iterable)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/csv.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: csv.DictReader.__init__(f: Iterable); call it with the wrong type.

typeshed contract: f is Iterable. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from csv import DictReader
try:
    DictReader(_W(), None)  # f: Iterable <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
