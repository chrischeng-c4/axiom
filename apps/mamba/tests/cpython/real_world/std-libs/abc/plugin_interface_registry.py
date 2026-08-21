# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "abc"
# dimension = "real_world"
# case = "plugin_interface_registry"
# subject = "abc.ABC"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_abc.py"
# status = "filled"
# ///
"""abc.ABC: a plugin system defines an ABC interface, enforces abstractmethod implementation on real plugins, and accepts a duck-typed third-party class via register()"""
import abc


# A real-world plugin contract: every plugin must declare a name and run().
class Plugin(abc.ABC):
    @abc.abstractmethod
    def name(self) -> str: ...
    @abc.abstractmethod
    def run(self, payload: str) -> str: ...


# An in-tree plugin implementing the full contract.
class UpperPlugin(Plugin):
    def name(self) -> str:
        return "upper"
    def run(self, payload: str) -> str:
        return payload.upper()


# A half-baked plugin: forgetting run() must be rejected at instantiation.
class BrokenPlugin(Plugin):
    def name(self) -> str:
        return "broken"


# A third-party class that already quacks like a plugin but cannot inherit;
# the host registers it as a virtual subclass.
class ThirdPartyReverser:
    def name(self) -> str:
        return "reverse"
    def run(self, payload: str) -> str:
        return payload[::-1]


Plugin.register(ThirdPartyReverser)


def load(plugin_cls):
    """Host loader: only accept classes recognized as Plugin."""
    assert issubclass(plugin_cls, Plugin), f"{plugin_cls.__name__} is not a Plugin"
    return plugin_cls()


# The broken plugin is recognized as a Plugin subclass but fails on construct.
_rejected = False
try:
    load(BrokenPlugin)
except TypeError:
    _rejected = True
assert _rejected, "incomplete plugin rejected at instantiation"

# Drive the registry over the valid in-tree and virtual third-party plugins.
registry = {}
for cls in (UpperPlugin, ThirdPartyReverser):
    plugin = load(cls)
    registry[plugin.name()] = plugin

assert set(registry) == {"upper", "reverse"}, f"registry keys: {sorted(registry)}"
assert registry["upper"].run("hello") == "HELLO", "upper plugin transforms input"
assert registry["reverse"].run("hello") == "olleh", "registered virtual plugin runs"
assert isinstance(registry["reverse"], Plugin), "virtual plugin isinstance Plugin"

print("plugin_interface_registry OK")
