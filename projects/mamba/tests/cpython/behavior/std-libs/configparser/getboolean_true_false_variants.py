# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "configparser"
# dimension = "behavior"
# case = "getboolean_true_false_variants"
# subject = "configparser.ConfigParser.getboolean"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""configparser.ConfigParser.getboolean: getboolean recognizes the full true/false vocabulary: yes/on/1/true -> True and no/off/0/false -> False"""
import configparser

cp = configparser.ConfigParser()
cp.read_string("[s]\na=yes\nb=no\nc=on\nd=off\ne=1\nf=0\ng=true\nh=false\n")

assert cp.getboolean("s", "a") is True, "yes=True"
assert cp.getboolean("s", "b") is False, "no=False"
assert cp.getboolean("s", "c") is True, "on=True"
assert cp.getboolean("s", "d") is False, "off=False"
assert cp.getboolean("s", "e") is True, "1=True"
assert cp.getboolean("s", "f") is False, "0=False"
assert cp.getboolean("s", "g") is True, "true=True"
assert cp.getboolean("s", "h") is False, "false=False"

print("getboolean_true_false_variants OK")
