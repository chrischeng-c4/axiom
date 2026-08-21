# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "configparser"
# dimension = "behavior"
# case = "read_file_accepts_line_iterable"
# subject = "configparser.ConfigParser.read_file"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_configparser.py"
# status = "filled"
# ///
"""configparser.ConfigParser.read_file: read_file accepts any iterable of lines (not just a file object) and populates sections; the section is then a member and its keys are readable"""
import configparser

cp = configparser.ConfigParser()
cp.read_file(["[Foo Bar]\n", "foo = newbar\n"])

assert "Foo Bar" in cp, "read_file iterable populates sections"
assert cp["Foo Bar"]["foo"] == "newbar", f"iterable value = {cp['Foo Bar']['foo']!r}"

print("read_file_accepts_line_iterable OK")
