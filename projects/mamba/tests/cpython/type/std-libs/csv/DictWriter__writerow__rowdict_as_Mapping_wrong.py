# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "csv"
# dimension = "type"
# case = "DictWriter__writerow__rowdict_as_Mapping_wrong"
# subject = "csv.DictWriter.writerow(rowdict: Mapping)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed rowdict"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/csv.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed rowdict
# mamba-strict-type: TypeError
"""Type wall: csv.DictWriter.writerow(rowdict: Mapping); call it with the wrong type.

typeshed contract: rowdict is Mapping. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from csv import DictWriter
obj = object.__new__(DictWriter)
try:
    obj.writerow(_W())  # rowdict: Mapping <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
