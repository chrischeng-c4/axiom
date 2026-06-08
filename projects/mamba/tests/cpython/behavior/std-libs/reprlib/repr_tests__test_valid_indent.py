# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "reprlib"
# dimension = "behavior"
# case = "repr_tests__test_valid_indent"
# subject = "cpython.test_reprlib.ReprTests.test_valid_indent"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_reprlib.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_reprlib.py::ReprTests::test_valid_indent
"""Auto-ported test: ReprTests::test_valid_indent (CPython 3.12 oracle)."""


import sys
import os
import shutil
import importlib
import importlib.util
import unittest
import textwrap
from test.support import verbose
from test.support.os_helper import create_empty_file
from reprlib import repr as r
from reprlib import Repr
from reprlib import recursive_repr


'\n  Test cases for the repr module\n  Nick Mathewson\n'

def nestedTuple(nesting):
    t = ()
    for i in range(nesting):
        t = (t,)
    return t

def write_file(path, text):
    with open(path, 'w', encoding='ASCII') as fp:
        fp.write(text)

class ClassWithRepr:

    def __init__(self, s):
        self.s = s

    def __repr__(self):
        return 'ClassWithRepr(%r)' % self.s

class ClassWithFailingRepr:

    def __repr__(self):
        raise Exception('This should be caught by Repr.repr_instance')

class MyContainer:
    """Helper class for TestRecursiveRepr"""

    def __init__(self, values):
        self.values = list(values)

    def append(self, value):
        self.values.append(value)

    @recursive_repr()
    def __repr__(self):
        return '<' + ', '.join(map(str, self.values)) + '>'

class MyContainer2(MyContainer):

    @recursive_repr('+++')
    def __repr__(self):
        return '<' + ', '.join(map(str, self.values)) + '>'

class MyContainer3:

    def __repr__(self):
        """Test document content"""
        pass
    wrapped = __repr__
    wrapper = recursive_repr()(wrapped)


# --- test body ---
test_cases = [{'object': (), 'tests': ((dict(indent=None), '()'), (dict(indent=False), '()'), (dict(indent=True), '()'), (dict(indent=0), '()'), (dict(indent=1), '()'), (dict(indent=4), '()'), (dict(indent=4, maxlevel=2), '()'), (dict(indent=''), '()'), (dict(indent='-->'), '()'), (dict(indent='....'), '()'))}, {'object': '', 'tests': ((dict(indent=None), "''"), (dict(indent=False), "''"), (dict(indent=True), "''"), (dict(indent=0), "''"), (dict(indent=1), "''"), (dict(indent=4), "''"), (dict(indent=4, maxlevel=2), "''"), (dict(indent=''), "''"), (dict(indent='-->'), "''"), (dict(indent='....'), "''"))}, {'object': [1, 'spam', {'eggs': True, 'ham': []}], 'tests': ((dict(indent=None), "                        [1, 'spam', {'eggs': True, 'ham': []}]"), (dict(indent=False), "                        [\n                        1,\n                        'spam',\n                        {\n                        'eggs': True,\n                        'ham': [],\n                        },\n                        ]"), (dict(indent=True), "                        [\n                         1,\n                         'spam',\n                         {\n                          'eggs': True,\n                          'ham': [],\n                         },\n                        ]"), (dict(indent=0), "                        [\n                        1,\n                        'spam',\n                        {\n                        'eggs': True,\n                        'ham': [],\n                        },\n                        ]"), (dict(indent=1), "                        [\n                         1,\n                         'spam',\n                         {\n                          'eggs': True,\n                          'ham': [],\n                         },\n                        ]"), (dict(indent=4), "                        [\n                            1,\n                            'spam',\n                            {\n                                'eggs': True,\n                                'ham': [],\n                            },\n                        ]"), (dict(indent=4, maxlevel=2), "                        [\n                            1,\n                            'spam',\n                            {\n                                'eggs': True,\n                                'ham': [],\n                            },\n                        ]"), (dict(indent=''), "                        [\n                        1,\n                        'spam',\n                        {\n                        'eggs': True,\n                        'ham': [],\n                        },\n                        ]"), (dict(indent='-->'), "                        [\n                        -->1,\n                        -->'spam',\n                        -->{\n                        -->-->'eggs': True,\n                        -->-->'ham': [],\n                        -->},\n                        ]"), (dict(indent='....'), "                        [\n                        ....1,\n                        ....'spam',\n                        ....{\n                        ........'eggs': True,\n                        ........'ham': [],\n                        ....},\n                        ]"))}, {'object': {1: 'two', b'three': [(4.5, 6.7), [set((8, 9)), frozenset((10, 11))]]}, 'tests': ((dict(indent=None), "                        {1: 'two', b'three': [(4.5, 6.7), [{8, 9}, frozenset({10, 11})]]}"), (dict(indent=False), "                        {\n                        1: 'two',\n                        b'three': [\n                        (\n                        4.5,\n                        6.7,\n                        ),\n                        [\n                        {\n                        8,\n                        9,\n                        },\n                        frozenset({\n                        10,\n                        11,\n                        }),\n                        ],\n                        ],\n                        }"), (dict(indent=True), "                        {\n                         1: 'two',\n                         b'three': [\n                          (\n                           4.5,\n                           6.7,\n                          ),\n                          [\n                           {\n                            8,\n                            9,\n                           },\n                           frozenset({\n                            10,\n                            11,\n                           }),\n                          ],\n                         ],\n                        }"), (dict(indent=0), "                        {\n                        1: 'two',\n                        b'three': [\n                        (\n                        4.5,\n                        6.7,\n                        ),\n                        [\n                        {\n                        8,\n                        9,\n                        },\n                        frozenset({\n                        10,\n                        11,\n                        }),\n                        ],\n                        ],\n                        }"), (dict(indent=1), "                        {\n                         1: 'two',\n                         b'three': [\n                          (\n                           4.5,\n                           6.7,\n                          ),\n                          [\n                           {\n                            8,\n                            9,\n                           },\n                           frozenset({\n                            10,\n                            11,\n                           }),\n                          ],\n                         ],\n                        }"), (dict(indent=4), "                        {\n                            1: 'two',\n                            b'three': [\n                                (\n                                    4.5,\n                                    6.7,\n                                ),\n                                [\n                                    {\n                                        8,\n                                        9,\n                                    },\n                                    frozenset({\n                                        10,\n                                        11,\n                                    }),\n                                ],\n                            ],\n                        }"), (dict(indent=4, maxlevel=2), "                        {\n                            1: 'two',\n                            b'three': [\n                                (...),\n                                [...],\n                            ],\n                        }"), (dict(indent=''), "                        {\n                        1: 'two',\n                        b'three': [\n                        (\n                        4.5,\n                        6.7,\n                        ),\n                        [\n                        {\n                        8,\n                        9,\n                        },\n                        frozenset({\n                        10,\n                        11,\n                        }),\n                        ],\n                        ],\n                        }"), (dict(indent='-->'), "                        {\n                        -->1: 'two',\n                        -->b'three': [\n                        -->-->(\n                        -->-->-->4.5,\n                        -->-->-->6.7,\n                        -->-->),\n                        -->-->[\n                        -->-->-->{\n                        -->-->-->-->8,\n                        -->-->-->-->9,\n                        -->-->-->},\n                        -->-->-->frozenset({\n                        -->-->-->-->10,\n                        -->-->-->-->11,\n                        -->-->-->}),\n                        -->-->],\n                        -->],\n                        }"), (dict(indent='....'), "                        {\n                        ....1: 'two',\n                        ....b'three': [\n                        ........(\n                        ............4.5,\n                        ............6.7,\n                        ........),\n                        ........[\n                        ............{\n                        ................8,\n                        ................9,\n                        ............},\n                        ............frozenset({\n                        ................10,\n                        ................11,\n                        ............}),\n                        ........],\n                        ....],\n                        }"))}]
for test_case in test_cases:
    for repr_settings, expected_repr in test_case['tests']:
        r = Repr()
        for attribute, value in repr_settings.items():
            setattr(r, attribute, value)
        resulting_repr = r.repr(test_case['object'])
        expected_repr = textwrap.dedent(expected_repr)

        assert resulting_repr == expected_repr
print("ReprTests::test_valid_indent: ok")
