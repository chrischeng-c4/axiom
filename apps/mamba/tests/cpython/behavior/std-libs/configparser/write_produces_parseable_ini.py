# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "configparser"
# dimension = "behavior"
# case = "write_produces_parseable_ini"
# subject = "configparser.ConfigParser.write"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_configparser.py"
# status = "filled"
# ///
"""configparser.ConfigParser.write: assigning a section dict then write(buf) emits parseable ini text containing the [section] header and 'key = value' lines"""
import configparser
import io

cp = configparser.ConfigParser()
cp["mysec"] = {"alpha": "1", "beta": "2"}

buf = io.StringIO()
cp.write(buf)
ini_text = buf.getvalue()

assert "[mysec]" in ini_text, "section in output"
assert "alpha = 1" in ini_text, "key=val in output"
assert "beta = 2" in ini_text, "second key=val in output"

# The emitted text round-trips back through the parser.
cp2 = configparser.ConfigParser()
cp2.read_string(ini_text)
assert cp2.get("mysec", "alpha") == "1", "round-trip alpha"
assert cp2.get("mysec", "beta") == "2", "round-trip beta"

print("write_produces_parseable_ini OK")
