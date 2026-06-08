# Atomic 267 pass conformance — argparse module (hasattr Argument
# Parser) + configparser module (hasattr ConfigParser/SafeConfig
# Parser) + csv module (hasattr reader/writer/DictReader/DictWriter/
# QUOTE_ALL/QUOTE_MINIMAL/QUOTE_NONE/Dialect/excel/excel_tab/unix_
# dialect/register_dialect/unregister_dialect/get_dialect/list_
# dialects + QUOTE_ALL==1, QUOTE_MINIMAL==0, QUOTE_NONE==3, reader
# of 'a,b,c'/'1,2,3' parses to two 3-cell rows, reader of one row
# parses to [['a','b','c']], reader of empty list is [], reader of
# '"a,b",c' parses to [['a,b', 'c']], excel.delimiter==',', excel_
# tab.delimiter=='\\t', 'excel' in list_dialects, get_dialect
# ('excel').delimiter==',') + getopt module (hasattr getopt/gnu_
# getopt/GetoptError + getopt(['-a', '1'], 'a:') extracts -a=1,
# getopt(['-a'], 'a') extracts -a='', getopt([], 'abc') returns
# ([], []), getopt with trailing positional preserves the rest,
# getopt(['--foo=bar'], '', ['foo=']) extracts --foo='bar',
# gnu_getopt with mixed positional).
# All asserts match between CPython 3.12 and mamba.
import argparse
import configparser
import csv
import getopt


_ledger: list[int] = []

# 1) argparse — hasattr ArgumentParser
assert hasattr(argparse, "ArgumentParser") == True; _ledger.append(1)

# 2) configparser — hasattr ConfigParser; SafeConfigParser was
#    removed in Python 3.12 — both runtimes report False
assert hasattr(configparser, "ConfigParser") == True; _ledger.append(1)
assert hasattr(configparser, "SafeConfigParser") == False; _ledger.append(1)

# 3) csv — hasattr surface
assert hasattr(csv, "reader") == True; _ledger.append(1)
assert hasattr(csv, "writer") == True; _ledger.append(1)
assert hasattr(csv, "DictReader") == True; _ledger.append(1)
assert hasattr(csv, "DictWriter") == True; _ledger.append(1)
assert hasattr(csv, "QUOTE_ALL") == True; _ledger.append(1)
assert hasattr(csv, "QUOTE_MINIMAL") == True; _ledger.append(1)
assert hasattr(csv, "QUOTE_NONE") == True; _ledger.append(1)
assert hasattr(csv, "Dialect") == True; _ledger.append(1)
assert hasattr(csv, "excel") == True; _ledger.append(1)
assert hasattr(csv, "excel_tab") == True; _ledger.append(1)
assert hasattr(csv, "unix_dialect") == True; _ledger.append(1)
assert hasattr(csv, "register_dialect") == True; _ledger.append(1)
assert hasattr(csv, "unregister_dialect") == True; _ledger.append(1)
assert hasattr(csv, "get_dialect") == True; _ledger.append(1)
assert hasattr(csv, "list_dialects") == True; _ledger.append(1)

# 4) csv — quote-style constants
assert csv.QUOTE_ALL == 1; _ledger.append(1)
assert csv.QUOTE_MINIMAL == 0; _ledger.append(1)
assert csv.QUOTE_NONE == 3; _ledger.append(1)

# 5) csv — reader parses CSV rows
assert list(csv.reader(["a,b,c", "1,2,3"])) == [["a", "b", "c"], ["1", "2", "3"]]; _ledger.append(1)
assert list(csv.reader(["a,b,c"])) == [["a", "b", "c"]]; _ledger.append(1)
assert list(csv.reader([])) == []; _ledger.append(1)
assert list(csv.reader(['"a,b",c'])) == [["a,b", "c"]]; _ledger.append(1)

# 6) csv — dialect attributes
assert csv.excel.delimiter == ","; _ledger.append(1)
assert csv.excel_tab.delimiter == "\t"; _ledger.append(1)
assert ("excel" in csv.list_dialects()) == True; _ledger.append(1)
assert csv.get_dialect("excel").delimiter == ","; _ledger.append(1)

# 7) getopt — hasattr surface
assert hasattr(getopt, "getopt") == True; _ledger.append(1)
assert hasattr(getopt, "gnu_getopt") == True; _ledger.append(1)
assert hasattr(getopt, "GetoptError") == True; _ledger.append(1)

# 8) getopt — parsing contracts
assert getopt.getopt(["-a", "1"], "a:")[0] == [("-a", "1")]; _ledger.append(1)
assert getopt.getopt(["-a"], "a")[0] == [("-a", "")]; _ledger.append(1)
assert getopt.getopt([], "abc") == ([], []); _ledger.append(1)
assert getopt.getopt(["-a", "1", "rest"], "a:") == ([("-a", "1")], ["rest"]); _ledger.append(1)
assert getopt.getopt(["--foo=bar"], "", ["foo="])[0] == [("--foo", "bar")]; _ledger.append(1)
assert getopt.gnu_getopt(["-a", "x", "extra"], "a:")[0] == [("-a", "x")]; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_argparse_configparser_csv_getopt_value_ops {sum(_ledger)} asserts")
