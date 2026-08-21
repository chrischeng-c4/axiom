# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "csv"
# dimension = "type"
# case = "Sniffer__sniff__sample_as_str_wrong"
# subject = "csv.Sniffer.sniff(sample: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/csv.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: csv.Sniffer.sniff(sample: str); call it with the wrong type.

typeshed contract: sample is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from csv import Sniffer
obj = object.__new__(Sniffer)
try:
    obj.sniff(12345)  # sample: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
