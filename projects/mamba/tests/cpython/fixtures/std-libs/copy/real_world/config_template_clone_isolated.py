# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "copy"
# dimension = "real_world"
# case = "config_template_clone_isolated"
# subject = "copy.deepcopy"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_copy.py"
# status = "filled"
# ///
"""copy.deepcopy: deepcopy a shared default-config template into a per-request config so mutating one request's nested settings never leaks into the template or sibling requests"""
import copy

# A shared default-config template with nested mutable settings.
DEFAULT_CONFIG = {
    "retries": 3,
    "headers": {"Accept": "application/json"},
    "endpoints": ["primary", "secondary"],
}


def new_request_config():
    # Each request gets a fully-independent clone of the template.
    return copy.deepcopy(DEFAULT_CONFIG)


# Request A tweaks its own nested settings.
cfg_a = new_request_config()
cfg_a["headers"]["Authorization"] = "Bearer token-a"
cfg_a["endpoints"].append("fallback-a")
cfg_a["retries"] = 5

# Request B is unaffected by A's mutations.
cfg_b = new_request_config()
assert cfg_b["headers"] == {"Accept": "application/json"}, "B headers isolated from A"
assert cfg_b["endpoints"] == ["primary", "secondary"], "B endpoints isolated from A"
assert cfg_b["retries"] == 3, "B retries isolated from A"

# The shared template itself is never mutated.
assert DEFAULT_CONFIG["headers"] == {"Accept": "application/json"}, "template headers intact"
assert DEFAULT_CONFIG["endpoints"] == ["primary", "secondary"], "template endpoints intact"
assert DEFAULT_CONFIG["retries"] == 3, "template retries intact"

print("config_template_clone_isolated OK")
