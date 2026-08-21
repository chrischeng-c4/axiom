# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pprint"
# dimension = "real_world"
# case = "config_dump_roundtrip"
# subject = "pprint.pformat"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_pprint.py"
# status = "filled"
# ///
"""pprint.pformat: a config-dump workflow: pformat a nested settings dict with sort_dicts/width/indent, assert the rendered block, and eval it back to the identical structure"""
import pprint

# A realistic nested settings document an app might dump to a log or a file.
config = {
    "server": {"host": "0.0.0.0", "port": 8080, "workers": 4},
    "logging": {"level": "info", "handlers": ["console", "file"]},
    "features": {"cache": True, "retry": 3, "timeout": 30.0},
}

# Render it with the default sorted keys, a narrow width and a 2-space indent
# so the deep structure wraps. The exact block is the CPython-3.12 oracle.
rendered = pprint.pformat(config, width=40, indent=2, sort_dicts=True)
assert rendered == (
    "{ 'features': { 'cache': True,\n"
    "                'retry': 3,\n"
    "                'timeout': 30.0},\n"
    "  'logging': { 'handlers': [ 'console',\n"
    "                             'file'],\n"
    "               'level': 'info'},\n"
    "  'server': { 'host': '0.0.0.0',\n"
    "              'port': 8080,\n"
    "              'workers': 4}}"
), rendered

# Sorting reordered the top-level keys away from insertion order.
assert list(config) == ["server", "logging", "features"]
assert rendered.index("'features'") < rendered.index("'logging'") \
    < rendered.index("'server'")

# The dump is a faithful literal: eval reconstructs the identical structure.
assert eval(rendered) == config

# sort_dicts=False preserves the original insertion order in the dump.
ordered = pprint.pformat(config, width=40, indent=2, sort_dicts=False)
assert ordered.index("'server'") < ordered.index("'logging'") \
    < ordered.index("'features'")
assert eval(ordered) == config
print("config_dump_roundtrip OK")
