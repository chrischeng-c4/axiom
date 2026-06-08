# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zoneinfo"
# dimension = "security"
# case = "untrusted_key_path_traversal_rejected"
# subject = "zoneinfo.ZoneInfo"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_zoneinfo/test_zoneinfo.py"
# status = "filled"
# ///
"""zoneinfo.ZoneInfo: untrusted zone keys with traversal/absolute components ('../etc/passwd', '/etc/passwd') are rejected, never escaping TZPATH to read arbitrary files"""
import zoneinfo

# An attacker-supplied "timezone" key must not be usable to read files
# outside TZPATH: traversal, absolute paths, and parent-escape sequences
# all raise ValueError before any file is opened.
hostile_keys = [
    "../etc/passwd",
    "/etc/passwd",
    "foo/../../bar",
    "../../../../etc/hosts",
]
for key in hostile_keys:
    raised = False
    try:
        zoneinfo.ZoneInfo(key)
    except ValueError:
        raised = True
    assert raised, "hostile key %r must be rejected with ValueError" % key
print("untrusted_key_path_traversal_rejected OK")
