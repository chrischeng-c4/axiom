# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "string"
# dimension = "behavior"
# case = "formatter_override_convert_field"
# subject = "string.Formatter"
# kind = "semantic"
# xfail = "string.Formatter subclassing relies on the format engine that is a silent dict-stub on mamba (repo-memory stdlib_stub_audit_2026_05_26)"
# mem_carveout = ""
# source = "Lib/test/test_string.py"
# status = "filled"
# ///
"""string.Formatter: overriding convert_field() adds a custom '!x' conversion while delegating others to super: XFormatter().format('{0!r}:{0!x}', 'foo', 'foo') == "'foo':None" """
import string


class XFormatter(string.Formatter):
    def convert_field(self, value, conversion):
        if conversion == "x":
            return None
        return super().convert_field(value, conversion)


assert XFormatter().format("{0!r}:{0!x}", "foo", "foo") == "'foo':None", "convert_field override"
print("formatter_override_convert_field OK")
