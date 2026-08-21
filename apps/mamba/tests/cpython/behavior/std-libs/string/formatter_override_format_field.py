# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "string"
# dimension = "behavior"
# case = "formatter_override_format_field"
# subject = "string.Formatter"
# kind = "semantic"
# xfail = "string.Formatter subclassing relies on the format engine that is a silent dict-stub on mamba (repo-memory stdlib_stub_audit_2026_05_26)"
# mem_carveout = ""
# source = "Lib/test/test_string.py"
# status = "filled"
# ///
"""string.Formatter: overriding format_field() can transform the value before formatting: CallFormatter calling the value -> format('*{0}*', lambda: 'result') == '*result*'"""
import string


class CallFormatter(string.Formatter):
    def format_field(self, value, format_spec):
        return format(value(), format_spec)


assert CallFormatter().format("*{0}*", lambda: "result") == "*result*", "format_field override"
print("formatter_override_format_field OK")
