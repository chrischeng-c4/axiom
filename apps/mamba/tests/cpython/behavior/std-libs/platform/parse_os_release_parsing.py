# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "platform"
# dimension = "behavior"
# case = "parse_os_release_parsing"
# subject = "platform._parse_os_release"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_platform.py"
# status = "filled"
# ///
"""platform._parse_os_release: _parse_os_release strips quotes, resolves shell escapes, skips comment/blank/malformed lines, and supplies default ID/NAME/PRETTY_NAME"""
import platform

FEDORA = (
    'NAME=Fedora\n'
    'VERSION="32 (Thirty Two)"\n'
    'ID=fedora\n'
    'VERSION_CODENAME=""\n'
)
info = platform._parse_os_release(FEDORA.splitlines())
assert info["NAME"] == "Fedora", "unquoted value"
assert info["ID"] == "fedora", "lowercase id"
assert info["VERSION_CODENAME"] == "", "empty quoted value -> empty string"
assert "ID_LIKE" not in info, "absent key is missing, not blank"

UBUNTU = (
    'NAME="Ubuntu"\n'
    'ID=ubuntu\n'
    'ID_LIKE=debian\n'
    'VERSION_CODENAME=focal\n'
)
info = platform._parse_os_release(UBUNTU.splitlines())
assert info["NAME"] == "Ubuntu", "quoted value stripped"
assert info["ID_LIKE"] == "debian", "id_like preserved"
assert info["VERSION_CODENAME"] == "focal", "bare value preserved"

# Comments, blanks, and malformed lines are ignored; quoting + escapes resolve.
TRICKY = (
    '\n'
    '# comment line\n'
    'ID_LIKE="egg spam viking"\n'
    'EMPTY=\n'
    "SINGLE_QUOTE='single'\n"
    'DOUBLE_QUOTE="double"\n'
    'QUOTES="double\\\'s"\n'
    'SPECIALS="\\$\\`\\\\\\\'\\""\n'
    '=invalid\n'
    'INVALID\n'
    'IN-VALID=value\n'
)
info = platform._parse_os_release(TRICKY.splitlines())
assert info["ID"] == "linux", "default ID when unspecified"
assert info["NAME"] == "Linux", "default NAME"
assert info["PRETTY_NAME"] == "Linux", "default PRETTY_NAME"
assert info["ID_LIKE"] == "egg spam viking", "spaces inside quotes kept"
assert info["EMPTY"] == "", "bare KEY= is empty string"
assert info["SINGLE_QUOTE"] == "single", "single quotes stripped"
assert info["DOUBLE_QUOTE"] == "double", "double quotes stripped"
assert info["QUOTES"] == "double's", "escaped apostrophe resolved"
assert info["SPECIALS"] == '$`\\\'"', "shell escapes resolved"
assert len(info["SPECIALS"]) == 5, "five resolved special chars"
assert "IN-VALID" not in info, "key with dash rejected"

print("parse_os_release_parsing OK")
