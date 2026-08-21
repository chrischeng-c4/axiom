# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "string"
# dimension = "security"
# case = "template_untrusted_no_attribute_leak"
# subject = "string.Template"
# kind = "semantic"
# xfail = "string.Template is a silent dict-stub on mamba; .safe_substitute() AttributeError (repo-memory stdlib_stub_audit_2026_05_26)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""string.Template: Template.safe_substitute over untrusted text with unknown $placeholders and a '$$' must not raise, must not pull values from outside the supplied mapping, and leaves unknown placeholders verbatim (no attribute/object leak)"""
import string

# A secret that lives only as a module global, never in the mapping.
SECRET = "do-not-leak"

# Untrusted user-controlled template text probing for unknown identifiers.
hostile = string.Template(
    "user=$username SECRET=$SECRET admin=${is_admin} literal=$$SECRET tail=$dangling"
)

# safe_substitute over a minimal trusted mapping must not raise and must
# leave every unsupplied placeholder verbatim — it cannot reach the module
# global SECRET or any object attribute.
out = hostile.safe_substitute(username="alice")
assert out == (
    "user=alice SECRET=$SECRET admin=${is_admin} literal=$SECRET tail=$dangling"
), out
assert SECRET not in out, "module global SECRET must not leak into the render"

# An invalid trailing '$' is also kept literally rather than raising.
trailer = string.Template("trusted=$ok then $")
assert trailer.safe_substitute(ok="yes") == "trusted=yes then $", "lone $ kept"

print("template_untrusted_no_attribute_leak OK")
