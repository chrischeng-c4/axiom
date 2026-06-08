# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "configparser"
# dimension = "behavior"
# case = "inline_comment_prefix_stripping"
# subject = "configparser.ConfigParser"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_configparser.py"
# status = "filled"
# ///
"""configparser.ConfigParser: with inline_comment_prefixes=(';','#','//') a prefix preceded by whitespace strips the trailing comment, while a prefix glued to the value is kept verbatim; without inline prefixes the marker stays part of the value"""
import configparser

src = (
    "[section]\n"
    "k1 = v1;still v1\n"
    "k2 = v2 ;a comment\n"
    "k3 = v3 ; also a comment\n"
    "k4 = v4;still v4 ;a comment\n"
    "k5 = v5;still v5; and still v5 ;a comment\n"
    "\n"
    "[multi]\n"
    "k1 = v1;still v1 #a comment ; yeah\n"
    "k2 = v2 // this is a comment ; continued\n"
    "k3 = v3;#//still v3# and still v3 ; a comment\n"
)

cfg = configparser.ConfigParser(inline_comment_prefixes=(";", "#", "//"))
cfg.read_string(src)

s = cfg["section"]
assert s["k1"] == "v1;still v1", f"k1 = {s['k1']!r}"  # no space before ; -> kept
assert s["k2"] == "v2", f"k2 = {s['k2']!r}"           # ' ;' strips comment
assert s["k3"] == "v3", f"k3 = {s['k3']!r}"
assert s["k4"] == "v4;still v4", f"k4 = {s['k4']!r}"
assert s["k5"] == "v5;still v5; and still v5", f"k5 = {s['k5']!r}"

m = cfg["multi"]
assert m["k1"] == "v1;still v1", f"multi k1 = {m['k1']!r}"
assert m["k2"] == "v2", f"multi k2 = {m['k2']!r}"  # ' //' strips comment
assert m["k3"] == "v3;#//still v3# and still v3", f"multi k3 = {m['k3']!r}"

# Without inline prefixes the marker stays part of the value.
plain = configparser.ConfigParser()
plain.read_string("[s]\nk = value ; not a comment\n")
assert plain["s"]["k"] == "value ; not a comment", f"plain = {plain['s']['k']!r}"

print("inline_comment_prefix_stripping OK")
