# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "configparser"
# dimension = "behavior"
# case = "read_dict_populates_from_nested_dict"
# subject = "configparser.ConfigParser.read_dict"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_configparser.py"
# status = "filled"
# ///
"""configparser.ConfigParser.read_dict: read_dict populates the parser from a nested {section: {key: value}} mapping; values come back as strings and coerce via getint"""
import configparser

cp = configparser.ConfigParser()
cp.read_dict({"db": {"host": "localhost", "port": "5432"}, "cache": {"url": "redis://"}})

assert cp.get("db", "host") == "localhost", f"host = {cp.get('db', 'host')!r}"
assert cp.getint("db", "port") == 5432, f"port = {cp.getint('db', 'port')!r}"
assert cp.get("cache", "url") == "redis://", f"url = {cp.get('cache', 'url')!r}"

print("read_dict_populates_from_nested_dict OK")
