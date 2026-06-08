# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "importlib"
# dimension = "real_world"
# case = "plugin_registry_dynamic_import"
# subject = "importlib.import_module"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_importlib"
# status = "filled"
# ///
"""importlib.import_module: a plugin loader resolves a batch of dotted module names from configuration via import_module, skipping unavailable plugins by catching ModuleNotFoundError, and collects the imported module objects into a registry the way an extensible app discovers optional backends"""
import importlib

# Configuration names a set of optional "plugin" backends. Some are real
# stdlib modules (available), one is not installed (must be skipped).
configured = [
    "json",
    "csv",
    "no_such_backend_plugin_xyzzy",
    "base64",
    "xml.etree.ElementTree",
]

registry = {}
skipped = []
for name in configured:
    try:
        registry[name] = importlib.import_module(name)
    except ModuleNotFoundError:
        skipped.append(name)

# Available plugins were loaded as real module objects keyed by their name.
assert set(registry) == {"json", "csv", "base64", "xml.etree.ElementTree"}, set(registry)
assert registry["json"].__name__ == "json", registry["json"].__name__
# A dotted name imports the leaf submodule object.
assert registry["xml.etree.ElementTree"].__name__ == "xml.etree.ElementTree"
# The unavailable backend was skipped, not fatal.
assert skipped == ["no_such_backend_plugin_xyzzy"], skipped

print("plugin_registry_dynamic_import OK", len(registry), "loaded")
