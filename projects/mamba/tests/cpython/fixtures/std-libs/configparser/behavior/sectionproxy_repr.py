# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "configparser"
# dimension = "behavior"
# case = "sectionproxy_repr"
# subject = "configparser.SectionProxy"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_configparser.py"
# status = "filled"
# ///
"""configparser.SectionProxy: repr of a section proxy is the fixed '<Section: name>' form"""
import configparser

cp = configparser.ConfigParser()
cp.read_string("[section]\nkey = value\n")

assert repr(cp["section"]) == "<Section: section>", f"proxy repr = {repr(cp['section'])!r}"

print("sectionproxy_repr OK")
