# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dataclasses"
# dimension = "behavior"
# case = "field_default_and_default_factory"
# subject = "dataclasses.field"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_dataclasses.py"
# status = "filled"
# ///
"""dataclasses.field: field(default=...) and field(default_factory=...) supply per-field defaults so a no-arg constructor yields the declared default values"""
import dataclasses


@dataclasses.dataclass
class Config:
    host: str = "localhost"
    port: int = dataclasses.field(default=8080)
    tags: list = dataclasses.field(default_factory=list)


c = Config()
assert c.host == "localhost", f"default host = {c.host!r}"
assert c.port == 8080, f"default port = {c.port!r}"
assert c.tags == [], f"default tags = {c.tags!r}"

print("field_default_and_default_factory OK")
