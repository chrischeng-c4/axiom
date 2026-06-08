# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "configparser"
# dimension = "surface"
# case = "api_section_proxy_is_present"
# subject = "configparser.SectionProxy"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""configparser.SectionProxy: api_section_proxy_is_present (surface)."""
import configparser

assert hasattr(configparser, "SectionProxy")
print("api_section_proxy_is_present OK")
