# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "gettext"
# dimension = "behavior"
# case = "plural_forms_internal_test_case__test_nested_condition_operator"
# subject = "cpython.test_gettext.PluralFormsInternalTestCase.test_nested_condition_operator"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_gettext.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_gettext.py::PluralFormsInternalTestCase::test_nested_condition_operator
"""Auto-ported test: PluralFormsInternalTestCase::test_nested_condition_operator (CPython 3.12 oracle)."""


import os
import base64
import gettext
import unittest
import unittest.mock
from functools import partial
from test import support
from test.support import os_helper


GNU_MO_DATA = b'3hIElQAAAAAJAAAAHAAAAGQAAAAAAAAArAAAAAAAAACsAAAAFQAAAK0AAAAjAAAAwwAAAKEAAADn\nAAAAMAAAAIkBAAAHAAAAugEAABYAAADCAQAAHAAAANkBAAALAAAA9gEAAEIBAAACAgAAFgAAAEUD\nAAAeAAAAXAMAAKEAAAB7AwAAMgAAAB0EAAAFAAAAUAQAABsAAABWBAAAIQAAAHIEAAAJAAAAlAQA\nAABSYXltb25kIEx1eHVyeSBZYWNoLXQAVGhlcmUgaXMgJXMgZmlsZQBUaGVyZSBhcmUgJXMgZmls\nZXMAVGhpcyBtb2R1bGUgcHJvdmlkZXMgaW50ZXJuYXRpb25hbGl6YXRpb24gYW5kIGxvY2FsaXph\ndGlvbgpzdXBwb3J0IGZvciB5b3VyIFB5dGhvbiBwcm9ncmFtcyBieSBwcm92aWRpbmcgYW4gaW50\nZXJmYWNlIHRvIHRoZSBHTlUKZ2V0dGV4dCBtZXNzYWdlIGNhdGFsb2cgbGlicmFyeS4AV2l0aCBj\nb250ZXh0BFRoZXJlIGlzICVzIGZpbGUAVGhlcmUgYXJlICVzIGZpbGVzAG11bGx1c2sAbXkgY29u\ndGV4dARudWRnZSBudWRnZQBteSBvdGhlciBjb250ZXh0BG51ZGdlIG51ZGdlAG51ZGdlIG51ZGdl\nAFByb2plY3QtSWQtVmVyc2lvbjogMi4wClBPLVJldmlzaW9uLURhdGU6IDIwMDMtMDQtMTEgMTQ6\nMzItMDQwMApMYXN0LVRyYW5zbGF0b3I6IEouIERhdmlkIEliYW5leiA8ai1kYXZpZEBub29zLmZy\nPgpMYW5ndWFnZS1UZWFtOiBYWCA8cHl0aG9uLWRldkBweXRob24ub3JnPgpNSU1FLVZlcnNpb246\nIDEuMApDb250ZW50LVR5cGU6IHRleHQvcGxhaW47IGNoYXJzZXQ9aXNvLTg4NTktMQpDb250ZW50\nLVRyYW5zZmVyLUVuY29kaW5nOiA4Yml0CkdlbmVyYXRlZC1CeTogcHlnZXR0ZXh0LnB5IDEuMQpQ\nbHVyYWwtRm9ybXM6IG5wbHVyYWxzPTI7IHBsdXJhbD1uIT0xOwoAVGhyb2F0d29iYmxlciBNYW5n\ncm92ZQBIYXkgJXMgZmljaGVybwBIYXkgJXMgZmljaGVyb3MAR3V2ZiB6YnFoeXIgY2ViaXZxcmYg\ndmFncmVhbmd2YmFueXZtbmd2YmEgbmFxIHlicG55dm1uZ3ZiYQpmaGNjYmVnIHNiZSBsYmhlIENs\nZ3ViYSBjZWJ0ZW56ZiBvbCBjZWJpdnF2YXQgbmEgdmFncmVzbnByIGdiIGd1ciBUQUgKdHJnZ3Jr\nZyB6cmZmbnRyIHBuZ255YnQgeXZvZW5lbC4ASGF5ICVzIGZpY2hlcm8gKGNvbnRleHQpAEhheSAl\ncyBmaWNoZXJvcyAoY29udGV4dCkAYmFjb24Ad2luayB3aW5rIChpbiAibXkgY29udGV4dCIpAHdp\nbmsgd2luayAoaW4gIm15IG90aGVyIGNvbnRleHQiKQB3aW5rIHdpbmsA\n'

GNU_MO_DATA_BAD_MAGIC_NUMBER = base64.b64encode(b'ABCD')

GNU_MO_DATA_BAD_MAJOR_VERSION = b'3hIElQAABQAGAAAAHAAAAEwAAAALAAAAfAAAAAAAAACoAAAAFQAAAKkAAAAjAAAAvwAAAKEAAADj\nAAAABwAAAIUBAAALAAAAjQEAAEUBAACZAQAAFgAAAN8CAAAeAAAA9gIAAKEAAAAVAwAABQAAALcD\nAAAJAAAAvQMAAAEAAAADAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAEAAAABQAAAAYAAAACAAAAAFJh\neW1vbmQgTHV4dXJ5IFlhY2gtdABUaGVyZSBpcyAlcyBmaWxlAFRoZXJlIGFyZSAlcyBmaWxlcwBU\naGlzIG1vZHVsZSBwcm92aWRlcyBpbnRlcm5hdGlvbmFsaXphdGlvbiBhbmQgbG9jYWxpemF0aW9u\nCnN1cHBvcnQgZm9yIHlvdXIgUHl0aG9uIHByb2dyYW1zIGJ5IHByb3ZpZGluZyBhbiBpbnRlcmZh\nY2UgdG8gdGhlIEdOVQpnZXR0ZXh0IG1lc3NhZ2UgY2F0YWxvZyBsaWJyYXJ5LgBtdWxsdXNrAG51\nZGdlIG51ZGdlAFByb2plY3QtSWQtVmVyc2lvbjogMi4wClBPLVJldmlzaW9uLURhdGU6IDIwMDAt\nMDgtMjkgMTI6MTktMDQ6MDAKTGFzdC1UcmFuc2xhdG9yOiBKLiBEYXZpZCBJYsOhw7FleiA8ai1k\nYXZpZEBub29zLmZyPgpMYW5ndWFnZS1UZWFtOiBYWCA8cHl0aG9uLWRldkBweXRob24ub3JnPgpN\nSU1FLVZlcnNpb246IDEuMApDb250ZW50LVR5cGU6IHRleHQvcGxhaW47IGNoYXJzZXQ9aXNvLTg4\nNTktMQpDb250ZW50LVRyYW5zZmVyLUVuY29kaW5nOiBub25lCkdlbmVyYXRlZC1CeTogcHlnZXR0\nZXh0LnB5IDEuMQpQbHVyYWwtRm9ybXM6IG5wbHVyYWxzPTI7IHBsdXJhbD1uIT0xOwoAVGhyb2F0\nd29iYmxlciBNYW5ncm92ZQBIYXkgJXMgZmljaGVybwBIYXkgJXMgZmljaGVyb3MAR3V2ZiB6YnFo\neXIgY2ViaXZxcmYgdmFncmVhbmd2YmFueXZtbmd2YmEgbmFxIHlicG55dm1uZ3ZiYQpmaGNjYmVn\nIHNiZSBsYmhlIENsZ3ViYSBjZWJ0ZW56ZiBvbCBjZWJpdnF2YXQgbmEgdmFncmVzbnByIGdiIGd1\nciBUQUgKdHJnZ3JrZyB6cmZmbnRyIHBuZ255YnQgeXZvZW5lbC4AYmFjb24Ad2luayB3aW5rAA==\n'

GNU_MO_DATA_BAD_MINOR_VERSION = b'3hIElQcAAAAGAAAAHAAAAEwAAAALAAAAfAAAAAAAAACoAAAAFQAAAKkAAAAjAAAAvwAAAKEAAADj\nAAAABwAAAIUBAAALAAAAjQEAAEUBAACZAQAAFgAAAN8CAAAeAAAA9gIAAKEAAAAVAwAABQAAALcD\nAAAJAAAAvQMAAAEAAAADAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAEAAAABQAAAAYAAAACAAAAAFJh\neW1vbmQgTHV4dXJ5IFlhY2gtdABUaGVyZSBpcyAlcyBmaWxlAFRoZXJlIGFyZSAlcyBmaWxlcwBU\naGlzIG1vZHVsZSBwcm92aWRlcyBpbnRlcm5hdGlvbmFsaXphdGlvbiBhbmQgbG9jYWxpemF0aW9u\nCnN1cHBvcnQgZm9yIHlvdXIgUHl0aG9uIHByb2dyYW1zIGJ5IHByb3ZpZGluZyBhbiBpbnRlcmZh\nY2UgdG8gdGhlIEdOVQpnZXR0ZXh0IG1lc3NhZ2UgY2F0YWxvZyBsaWJyYXJ5LgBtdWxsdXNrAG51\nZGdlIG51ZGdlAFByb2plY3QtSWQtVmVyc2lvbjogMi4wClBPLVJldmlzaW9uLURhdGU6IDIwMDAt\nMDgtMjkgMTI6MTktMDQ6MDAKTGFzdC1UcmFuc2xhdG9yOiBKLiBEYXZpZCBJYsOhw7FleiA8ai1k\nYXZpZEBub29zLmZyPgpMYW5ndWFnZS1UZWFtOiBYWCA8cHl0aG9uLWRldkBweXRob24ub3JnPgpN\nSU1FLVZlcnNpb246IDEuMApDb250ZW50LVR5cGU6IHRleHQvcGxhaW47IGNoYXJzZXQ9aXNvLTg4\nNTktMQpDb250ZW50LVRyYW5zZmVyLUVuY29kaW5nOiBub25lCkdlbmVyYXRlZC1CeTogcHlnZXR0\nZXh0LnB5IDEuMQpQbHVyYWwtRm9ybXM6IG5wbHVyYWxzPTI7IHBsdXJhbD1uIT0xOwoAVGhyb2F0\nd29iYmxlciBNYW5ncm92ZQBIYXkgJXMgZmljaGVybwBIYXkgJXMgZmljaGVyb3MAR3V2ZiB6YnFo\neXIgY2ViaXZxcmYgdmFncmVhbmd2YmFueXZtbmd2YmEgbmFxIHlicG55dm1uZ3ZiYQpmaGNjYmVn\nIHNiZSBsYmhlIENsZ3ViYSBjZWJ0ZW56ZiBvbCBjZWJpdnF2YXQgbmEgdmFncmVzbnByIGdiIGd1\nciBUQUgKdHJnZ3JrZyB6cmZmbnRyIHBuZ255YnQgeXZvZW5lbC4AYmFjb24Ad2luayB3aW5rAA==\n'

GNU_MO_DATA_CORRUPT = base64.b64encode(bytes([222, 18, 4, 149, 0, 0, 0, 0, 1, 0, 0, 0, 28, 0, 0, 0, 36, 0, 0, 0, 0, 0, 0, 0, 44, 0, 0, 0, 3, 0, 0, 0, 44, 0, 0, 0, 3, 0, 0, 0, 255, 255, 255, 255, 102, 111, 111, 0, 98, 97, 114, 0]))

UMO_DATA = b'3hIElQAAAAADAAAAHAAAADQAAAAAAAAAAAAAAAAAAABMAAAABAAAAE0AAAAQAAAAUgAAAA8BAABj\nAAAABAAAAHMBAAAWAAAAeAEAAABhYsOeAG15Y29udGV4dMOeBGFiw54AUHJvamVjdC1JZC1WZXJz\naW9uOiAyLjAKUE8tUmV2aXNpb24tRGF0ZTogMjAwMy0wNC0xMSAxMjo0Mi0wNDAwCkxhc3QtVHJh\nbnNsYXRvcjogQmFycnkgQS4gV0Fyc2F3IDxiYXJyeUBweXRob24ub3JnPgpMYW5ndWFnZS1UZWFt\nOiBYWCA8cHl0aG9uLWRldkBweXRob24ub3JnPgpNSU1FLVZlcnNpb246IDEuMApDb250ZW50LVR5\ncGU6IHRleHQvcGxhaW47IGNoYXJzZXQ9dXRmLTgKQ29udGVudC1UcmFuc2Zlci1FbmNvZGluZzog\nN2JpdApHZW5lcmF0ZWQtQnk6IG1hbnVhbGx5CgDCpHl6AMKkeXogKGNvbnRleHQgdmVyc2lvbikA\n'

MMO_DATA = b'3hIElQAAAAABAAAAHAAAACQAAAADAAAALAAAAAAAAAA4AAAAeAEAADkAAAABAAAAAAAAAAAAAAAA\nUHJvamVjdC1JZC1WZXJzaW9uOiBObyBQcm9qZWN0IDAuMApQT1QtQ3JlYXRpb24tRGF0ZTogV2Vk\nIERlYyAxMSAwNzo0NDoxNSAyMDAyClBPLVJldmlzaW9uLURhdGU6IDIwMDItMDgtMTQgMDE6MTg6\nNTgrMDA6MDAKTGFzdC1UcmFuc2xhdG9yOiBKb2huIERvZSA8amRvZUBleGFtcGxlLmNvbT4KSmFu\nZSBGb29iYXIgPGpmb29iYXJAZXhhbXBsZS5jb20+Ckxhbmd1YWdlLVRlYW06IHh4IDx4eEBleGFt\ncGxlLmNvbT4KTUlNRS1WZXJzaW9uOiAxLjAKQ29udGVudC1UeXBlOiB0ZXh0L3BsYWluOyBjaGFy\nc2V0PWlzby04ODU5LTE1CkNvbnRlbnQtVHJhbnNmZXItRW5jb2Rpbmc6IHF1b3RlZC1wcmludGFi\nbGUKR2VuZXJhdGVkLUJ5OiBweWdldHRleHQucHkgMS4zCgA=\n'

LOCALEDIR = os.path.join('xx', 'LC_MESSAGES')

MOFILE = os.path.join(LOCALEDIR, 'gettext.mo')

MOFILE_BAD_MAGIC_NUMBER = os.path.join(LOCALEDIR, 'gettext_bad_magic_number.mo')

MOFILE_BAD_MAJOR_VERSION = os.path.join(LOCALEDIR, 'gettext_bad_major_version.mo')

MOFILE_BAD_MINOR_VERSION = os.path.join(LOCALEDIR, 'gettext_bad_minor_version.mo')

MOFILE_CORRUPT = os.path.join(LOCALEDIR, 'gettext_corrupt.mo')

UMOFILE = os.path.join(LOCALEDIR, 'ugettext.mo')

MMOFILE = os.path.join(LOCALEDIR, 'metadata.mo')

def reset_gettext():
    gettext._localedirs.clear()
    gettext._current_domain = 'messages'
    gettext._translations.clear()

GNU_MO_DATA_ISSUE_17898 = b'3hIElQAAAAABAAAAHAAAACQAAAAAAAAAAAAAAAAAAAAsAAAAggAAAC0AAAAAUGx1cmFsLUZvcm1z\nOiBucGx1cmFscz0yOyBwbHVyYWw9KG4gIT0gMSk7CiMtIy0jLSMtIyAgbWVzc2FnZXMucG8gKEVk\nWCBTdHVkaW8pICAjLSMtIy0jLSMKQ29udGVudC1UeXBlOiB0ZXh0L3BsYWluOyBjaGFyc2V0PVVU\nRi04CgA=\n'

class DummyGNUTranslations(gettext.GNUTranslations):

    def foo(self):
        return 'foo'

b'\n# Dummy translation for the Python test_gettext.py module.\n# Copyright (C) 2001 Python Software Foundation\n# Barry Warsaw <barry@python.org>, 2000.\n#\nmsgid ""\nmsgstr ""\n"Project-Id-Version: 2.0\n"\n"PO-Revision-Date: 2003-04-11 14:32-0400\n"\n"Last-Translator: J. David Ibanez <j-david@noos.fr>\n"\n"Language-Team: XX <python-dev@python.org>\n"\n"MIME-Version: 1.0\n"\n"Content-Type: text/plain; charset=iso-8859-1\n"\n"Content-Transfer-Encoding: 8bit\n"\n"Generated-By: pygettext.py 1.1\n"\n"Plural-Forms: nplurals=2; plural=n!=1;\n"\n\n#: test_gettext.py:19 test_gettext.py:25 test_gettext.py:31 test_gettext.py:37\n#: test_gettext.py:51 test_gettext.py:80 test_gettext.py:86 test_gettext.py:92\n#: test_gettext.py:98\nmsgid "nudge nudge"\nmsgstr "wink wink"\n\nmsgctxt "my context"\nmsgid "nudge nudge"\nmsgstr "wink wink (in "my context")"\n\nmsgctxt "my other context"\nmsgid "nudge nudge"\nmsgstr "wink wink (in "my other context")"\n\n#: test_gettext.py:16 test_gettext.py:22 test_gettext.py:28 test_gettext.py:34\n#: test_gettext.py:77 test_gettext.py:83 test_gettext.py:89 test_gettext.py:95\nmsgid "albatross"\nmsgstr ""\n\n#: test_gettext.py:18 test_gettext.py:24 test_gettext.py:30 test_gettext.py:36\n#: test_gettext.py:79 test_gettext.py:85 test_gettext.py:91 test_gettext.py:97\nmsgid "Raymond Luxury Yach-t"\nmsgstr "Throatwobbler Mangrove"\n\n#: test_gettext.py:17 test_gettext.py:23 test_gettext.py:29 test_gettext.py:35\n#: test_gettext.py:56 test_gettext.py:78 test_gettext.py:84 test_gettext.py:90\n#: test_gettext.py:96\nmsgid "mullusk"\nmsgstr "bacon"\n\n#: test_gettext.py:40 test_gettext.py:101\nmsgid ""\n"This module provides internationalization and localization\n"\n"support for your Python programs by providing an interface to the GNU\n"\n"gettext message catalog library."\nmsgstr ""\n"Guvf zbqhyr cebivqrf vagreangvbanyvmngvba naq ybpnyvmngvba\n"\n"fhccbeg sbe lbhe Clguba cebtenzf ol cebivqvat na vagresnpr gb gur TAH\n"\n"trggrkg zrffntr pngnybt yvoenel."\n\n# Manually added, as neither pygettext nor xgettext support plural forms\n# in Python.\nmsgid "There is %s file"\nmsgid_plural "There are %s files"\nmsgstr[0] "Hay %s fichero"\nmsgstr[1] "Hay %s ficheros"\n\n# Manually added, as neither pygettext nor xgettext support plural forms\n# and context in Python.\nmsgctxt "With context"\nmsgid "There is %s file"\nmsgid_plural "There are %s files"\nmsgstr[0] "Hay %s fichero (context)"\nmsgstr[1] "Hay %s ficheros (context)"\n'

b'\n# Dummy translation for the Python test_gettext.py module.\n# Copyright (C) 2001 Python Software Foundation\n# Barry Warsaw <barry@python.org>, 2000.\n#\nmsgid ""\nmsgstr ""\n"Project-Id-Version: 2.0\n"\n"PO-Revision-Date: 2003-04-11 12:42-0400\n"\n"Last-Translator: Barry A. WArsaw <barry@python.org>\n"\n"Language-Team: XX <python-dev@python.org>\n"\n"MIME-Version: 1.0\n"\n"Content-Type: text/plain; charset=utf-8\n"\n"Content-Transfer-Encoding: 7bit\n"\n"Generated-By: manually\n"\n\n#: nofile:0\nmsgid "ab\xc3\x9e"\nmsgstr "\xc2\xa4yz"\n\n#: nofile:1\nmsgctxt "mycontext\xc3\x9e"\nmsgid "ab\xc3\x9e"\nmsgstr "\xc2\xa4yz (context version)"\n'

b'\nmsgid ""\nmsgstr ""\n"Project-Id-Version: No Project 0.0\n"\n"POT-Creation-Date: Wed Dec 11 07:44:15 2002\n"\n"PO-Revision-Date: 2002-08-14 01:18:58+00:00\n"\n"Last-Translator: John Doe <jdoe@example.com>\n"\n"Jane Foobar <jfoobar@example.com>\n"\n"Language-Team: xx <xx@example.com>\n"\n"MIME-Version: 1.0\n"\n"Content-Type: text/plain; charset=iso-8859-15\n"\n"Content-Transfer-Encoding: quoted-printable\n"\n"Generated-By: pygettext.py 1.3\n"\n'

b'\n# test file for http://bugs.python.org/issue17898\nmsgid ""\nmsgstr ""\n"Plural-Forms: nplurals=2; plural=(n != 1);\n"\n"#-#-#-#-#  messages.po (EdX Studio)  #-#-#-#-#\n"\n"Content-Type: text/plain; charset=UTF-8\n"\n'


# --- test body ---

assert gettext.c2py('n?1?2:3:4')(0) == 4

assert gettext.c2py('n?1?2:3:4')(1) == 2

assert gettext.c2py('n?1:3?4:5')(0) == 4

assert gettext.c2py('n?1:3?4:5')(1) == 1
print("PluralFormsInternalTestCase::test_nested_condition_operator: ok")
