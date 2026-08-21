# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "dataclasses"
# dimension = "behavior"
# case = "field_defaults_applied"
# subject = "dataclasses.dataclass"
# kind = "semantic"
# xfail = "mamba does not synthesize __init__ defaults (repo memory project_mamba_class_machinery_silent_divergences)"
# mem_carveout = ""
# source = "surface.py"
# status = "filled"
# ///
"""dataclasses.dataclass: annotated defaults (host/port/debug) are applied when omitted and overridable positionally or by keyword"""
from dataclasses import dataclass


@dataclass
class Config:
    host: str = "localhost"
    port: int = 8080
    debug: bool = False


# All defaults applied when omitted.
cfg = Config()
assert cfg.host == "localhost", f"host = {cfg.host!r}"
assert cfg.port == 8080, f"port = {cfg.port!r}"
assert cfg.debug is False, f"debug = {cfg.debug!r}"

# Overridable by keyword (leaving the rest defaulted).
cfg2 = Config(host="example.com", port=443)
assert cfg2.host == "example.com", f"host2 = {cfg2.host!r}"
assert cfg2.port == 443, f"port2 = {cfg2.port!r}"
assert cfg2.debug is False, f"debug2 = {cfg2.debug!r}"
print("field_defaults_applied OK")
