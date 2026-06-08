# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "string"
# dimension = "real_world"
# case = "config_template_rendering"
# subject = "string.Template"
# kind = "semantic"
# xfail = "string.Template is a silent dict-stub on mamba; .substitute()/.safe_substitute() AttributeError (repo-memory stdlib_stub_audit_2026_05_26)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""string.Template: a config/templating consumer renders a multi-line settings file from a string.Template with $named and ${braced} fields and $$ literals, then safe_substitute leaves an optional placeholder when its value is absent"""
import string

# An end-user config-rendering flow: one Template drives a multi-line
# settings file mixing $named, ${braced}, and $$ literals.
config_tmpl = string.Template(
    "[server]\n"
    "host = $host\n"
    "port = ${port}\n"
    "url = http://$host:${port}/${path}\n"
    "currency = $$USD\n"
    "log_level = $log_level\n"
)

settings = {
    "host": "example.com",
    "port": 8080,
    "path": "api/v1",
    "log_level": "INFO",
}
rendered = config_tmpl.substitute(settings)
assert "host = example.com" in rendered, rendered
assert "port = 8080" in rendered, rendered
assert "url = http://example.com:8080/api/v1" in rendered, rendered
assert "currency = $USD" in rendered, rendered
assert "log_level = INFO" in rendered, rendered

# safe_substitute leaves an optional placeholder when its value is absent.
partial = config_tmpl.safe_substitute(host="h", port=1, path="p")
assert "$log_level" in partial, partial
assert "host = h" in partial, partial

print("config_template_rendering OK")
