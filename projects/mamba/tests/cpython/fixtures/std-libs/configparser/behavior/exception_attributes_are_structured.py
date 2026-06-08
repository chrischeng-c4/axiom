# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "configparser"
# dimension = "behavior"
# case = "exception_attributes_are_structured"
# subject = "configparser.DuplicateOptionError"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_configparser.py"
# status = "filled"
# ///
"""configparser.DuplicateOptionError: configparser exceptions carry structured attributes, not just a message: DuplicateOptionError exposes section/option/source/lineno and a canonical str, InterpolationDepthError keeps option/section/args, and ParsingError accepts source positionally or by keyword"""
import configparser

# DuplicateOptionError carries section/option and unset source/lineno as None.
doe = configparser.DuplicateOptionError("sect", "opt")
assert doe.section == "sect", f"DuplicateOptionError.section = {doe.section!r}"
assert doe.option == "opt", f"DuplicateOptionError.option = {doe.option!r}"
assert doe.source is None and doe.lineno is None, "unset source/lineno are None"
assert doe.args == ("sect", "opt", None, None), f"args = {doe.args!r}"
assert str(doe) == "Option 'opt' in section 'sect' already exists", f"str = {str(doe)!r}"

# InterpolationDepthError keeps option/section and the raw value in args.
ide = configparser.InterpolationDepthError("opt", "sect", "rawval")
assert ide.args == ("opt", "sect", "rawval"), f"depth args = {ide.args!r}"
assert ide.option == "opt" and ide.section == "sect", "depth option/section"

# ParsingError needs a source; accepts it positionally or by keyword.
assert configparser.ParsingError(source="src").source == "src", "keyword source"
assert configparser.ParsingError("pos").source == "pos", "positional source"

print("exception_attributes_are_structured OK")
