# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "textwrap"
# dimension = "real_world"
# case = "reflow_paragraph_with_indent_prefix"
# subject = "textwrap"
# kind = "semantic"
# xfail = "textwrap.fill is a silent stub under mamba — paragraph reflow does not wrap (repo memory project-mamba-stdlib-stub-audit-2026-05-26)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""textwrap: a downstream consumer dedents a raw docstring, fills it to a column, then indents the result as a quoted block — asserting each stage matches CPython"""
import textwrap

# A help/usage string captured as an over-indented raw block (the shape you get
# from a triple-quoted docstring). The end-user flow: dedent it, reflow the body
# paragraph to a fixed column, then render it as a "| "-quoted block.
raw = (
    "\n"
    "        Usage: tool [options] FILE\n"
    "\n"
    "        This command processes the FILE according to the supplied options and\n"
    "        writes the formatted result to standard output for downstream tools.\n"
)

# Stage 1: dedent strips the common 8-space prefix.
body = textwrap.dedent(raw).strip()
paragraphs = body.split("\n\n")
assert len(paragraphs) == 2, f"paragraphs = {paragraphs!r}"
assert paragraphs[0] == "Usage: tool [options] FILE", f"usage line = {paragraphs[0]!r}"

# Stage 2: reflow the body paragraph to a 40-column width.
filled = textwrap.fill(paragraphs[1], width=40)
lines = filled.split("\n")
assert len(lines) > 1, f"expected reflow into multiple lines, got {lines!r}"
assert all(len(line) <= 40 for line in lines), f"every line within 40 cols = {lines!r}"

# Stage 3: render as a quoted block.
quoted = textwrap.indent(filled, "    | ")
assert all(qline.startswith("    | ") for qline in quoted.split("\n")), (
    f"every quoted line prefixed = {quoted!r}"
)
assert quoted.split("\n")[0] == "    | This command processes the FILE", (
    f"first quoted line = {quoted.split(chr(10))[0]!r}"
)
print("reflow_paragraph_with_indent_prefix OK")
