# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "configparser"
# dimension = "behavior"
# case = "read_string_populates_sections"
# subject = "configparser.ConfigParser.read_string"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_configparser.py"
# status = "filled"
# ///
"""configparser.ConfigParser.read_string: read_string parses an ini blob; sections() lists non-DEFAULT sections (DEFAULT excluded), has_section / has_option / options reflect the parsed content, and get returns the raw string value"""
import configparser

cp = configparser.ConfigParser()
cp.read_string(
    "[section1]\n"
    "key1 = value1\n"
    "key2 = 42\n"
    "\n"
    "[section2]\n"
    "name = Alice\n"
)

secs = cp.sections()
assert isinstance(secs, list), f"sections type = {type(secs)!r}"
assert "section1" in secs, "section1 present"
assert "section2" in secs, "section2 present"
assert "DEFAULT" not in secs, "DEFAULT not in sections()"

assert cp.has_section("section1"), "has_section true"
assert not cp.has_section("nonexistent"), "has_section false"

assert cp.has_option("section1", "key1"), "has_option true"
assert not cp.has_option("section1", "nokey"), "has_option false"

opts = cp.options("section1")
assert "key1" in opts, "key1 in options"
assert "key2" in opts, "key2 in options"

assert cp.get("section1", "key1") == "value1", f"get = {cp.get('section1', 'key1')!r}"

print("read_string_populates_sections OK")
