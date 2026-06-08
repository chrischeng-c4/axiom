# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tomllib"
# dimension = "real_world"
# case = "app_config_load_and_query"
# subject = "tomllib.loads"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_tomllib/test_misc.py"
# status = "filled"
# ///
"""tomllib.loads: an application loads a realistic pyproject-style config (sections, arrays of tables, inline tables, typed scalars) and queries server host/port, a list of products, and a nested build setting, asserting each value"""
import tomllib

CONFIG = """
title = "demo service"
debug = false

[server]
host = "0.0.0.0"
port = 8080
workers = 4
limits = {max_body = 1_048_576, timeout = 30.0}

[database]
host = "localhost"
port = 5432

[build]
[build.cache]
enabled = true
ttl = 3600

[[products]]
name = "Hammer"
price = 9.99

[[products]]
name = "Wrench"
price = 14.99
"""

cfg = tomllib.loads(CONFIG)

# Top-level scalars.
assert cfg["title"] == "demo service", f"title = {cfg['title']!r}"
assert cfg["debug"] is False, f"debug = {cfg['debug']!r}"

# Server section, including an inline table.
server = cfg["server"]
assert server["host"] == "0.0.0.0", f"host = {server['host']!r}"
assert server["port"] == 8080, f"port = {server['port']!r}"
assert server["workers"] == 4, f"workers = {server['workers']!r}"
assert server["limits"]["max_body"] == 1048576, f"max_body = {server['limits']['max_body']!r}"
assert server["limits"]["timeout"] == 30.0, f"timeout = {server['limits']['timeout']!r}"

# A second section parses independently.
assert cfg["database"]["port"] == 5432, f"db port = {cfg['database']['port']!r}"

# Nested build.cache table.
assert cfg["build"]["cache"]["enabled"] is True, f"cache enabled = {cfg['build']['cache']['enabled']!r}"
assert cfg["build"]["cache"]["ttl"] == 3600, f"cache ttl = {cfg['build']['cache']['ttl']!r}"

# Array of tables: aggregate over a list of dicts.
products = cfg["products"]
assert [p["name"] for p in products] == ["Hammer", "Wrench"], f"products = {products!r}"
total = sum(p["price"] for p in products)
assert abs(total - 24.98) < 1e-9, f"total price = {total!r}"

print("app_config_load_and_query OK")
