# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "string"
# dimension = "behavior"
# case = "formatter_override_parse"
# subject = "string.Formatter"
# kind = "semantic"
# xfail = "string.Formatter subclassing relies on the format engine that is a silent dict-stub on mamba (repo-memory stdlib_stub_audit_2026_05_26)"
# mem_carveout = ""
# source = "Lib/test/test_string.py"
# status = "filled"
# ///
"""string.Formatter: overriding parse() defines a custom '|'-delimited field syntax: BarFormatter().format('*|+0:^10s|*', 'foo') == '*   foo    *'"""
import string


class BarFormatter(string.Formatter):
    def parse(self, format_string):
        for field in format_string.split("|"):
            if field[0] == "+":
                field_name, _, format_spec = field[1:].partition(":")
                yield ("", field_name, format_spec, None)
            else:
                yield (field, None, None, None)


assert BarFormatter().format("*|+0:^10s|*", "foo") == "*   foo    *", "parse override"
print("formatter_override_parse OK")
