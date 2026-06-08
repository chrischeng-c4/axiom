# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "csv"
# dimension = "behavior"
# case = "test_sniffer__test_guess_quote_and_delimiter"
# subject = "cpython.test_csv.TestSniffer.test_guess_quote_and_delimiter"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_csv.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_csv.py::TestSniffer::test_guess_quote_and_delimiter
"""Auto-ported test: TestSniffer::test_guess_quote_and_delimiter (CPython 3.12 oracle)."""


import copy
import sys
import unittest
from io import StringIO
from tempfile import TemporaryFile
import csv
import gc
import pickle
from test import support
from test.support import warnings_helper, import_helper, check_disallow_instantiation
from itertools import permutations
from textwrap import dedent
from collections import OrderedDict


class BadIterable:

    def __iter__(self):
        raise OSError

class EscapedExcel(csv.excel):
    quoting = csv.QUOTE_NONE
    escapechar = '\\'

class QuotedEscapedExcel(csv.excel):
    quoting = csv.QUOTE_NONNUMERIC
    escapechar = '\\'

class NUL:

    def write(s, *args):
        pass
    writelines = write


# --- test body ---
sample1 = "Harry's, Arlington Heights, IL, 2/1/03, Kimi Hayes\nShark City, Glendale Heights, IL, 12/28/02, Prezence\nTommy's Place, Blue Island, IL, 12/28/02, Blue Sunday/White Crow\nStonecutters Seafood and Chop House, Lemont, IL, 12/19/02, Week Back\n"
sample2 = "'Harry''s':'Arlington Heights':'IL':'2/1/03':'Kimi Hayes'\n'Shark City':'Glendale Heights':'IL':'12/28/02':'Prezence'\n'Tommy''s Place':'Blue Island':'IL':'12/28/02':'Blue Sunday/White Crow'\n'Stonecutters ''Seafood'' and Chop House':'Lemont':'IL':'12/19/02':'Week Back'\n"
header1 = '"venue","city","state","date","performers"\n'
sample3 = '05/05/03?05/05/03?05/05/03?05/05/03?05/05/03?05/05/03\n05/05/03?05/05/03?05/05/03?05/05/03?05/05/03?05/05/03\n05/05/03?05/05/03?05/05/03?05/05/03?05/05/03?05/05/03\n'
sample4 = '2147483648;43.0e12;17;abc;def\n147483648;43.0e2;17;abc;def\n47483648;43.0;170;abc;def\n'
sample5 = 'aaa\tbbb\r\nAAA\t\r\nBBB\t\r\n'
sample6 = 'a|b|c\r\nd|e|f\r\n'
sample7 = "'a'|'b'|'c'\r\n'd'|e|f\r\n"
header2 = '"venue"+"city"+"state"+"date"+"performers"\n'
sample8 = "Harry's+ Arlington Heights+ IL+ 2/1/03+ Kimi Hayes\nShark City+ Glendale Heights+ IL+ 12/28/02+ Prezence\nTommy's Place+ Blue Island+ IL+ 12/28/02+ Blue Sunday/White Crow\nStonecutters Seafood and Chop House+ Lemont+ IL+ 12/19/02+ Week Back\n"
sample9 = "'Harry''s'+ Arlington Heights'+ 'IL'+ '2/1/03'+ 'Kimi Hayes'\n'Shark City'+ Glendale Heights'+' IL'+ '12/28/02'+ 'Prezence'\n'Tommy''s Place'+ Blue Island'+ 'IL'+ '12/28/02'+ 'Blue Sunday/White Crow'\n'Stonecutters ''Seafood'' and Chop House'+ 'Lemont'+ 'IL'+ '12/19/02'+ 'Week Back'\n"
sample10 = dedent('\n                        abc,def\n                        ghijkl,mno\n                        ghi,jkl\n                        ')
sample11 = dedent('\n                        abc,def\n                        ghijkl,mnop\n                        ghi,jkl\n                         ')
sample12 = dedent('"time","forces"\n                        1,1.5\n                        0.5,5+0j\n                        0,0\n                        1+1j,6\n                        ')
sample13 = dedent('"time","forces"\n                        0,0\n                        1,2\n                        a,b\n                        ')
sample14 = 'abc\x00def\nghijkl\x00mno\nghi\x00jkl\n'
sniffer = csv.Sniffer()
for header in (";'123;4';", "'123;4';", ";'123;4'", "'123;4'"):
    dialect = sniffer.sniff(header, ',;')

    assert dialect.delimiter == ';'

    assert dialect.quotechar == "'"

    assert dialect.doublequote is False

    assert dialect.skipinitialspace is False
print("TestSniffer::test_guess_quote_and_delimiter: ok")
