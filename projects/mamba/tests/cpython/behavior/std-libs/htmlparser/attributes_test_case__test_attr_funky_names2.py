# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "htmlparser"
# dimension = "behavior"
# case = "attributes_test_case__test_attr_funky_names2"
# subject = "cpython.test_htmlparser.AttributesTestCase.test_attr_funky_names2"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_htmlparser.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_htmlparser.py::AttributesTestCase::test_attr_funky_names2
"""Auto-ported test: AttributesTestCase::test_attr_funky_names2 (CPython 3.12 oracle)."""


import html.parser
import pprint
import unittest
from unittest.mock import patch


'Tests for HTMLParser.py.'

class EventCollector(html.parser.HTMLParser):

    def __init__(self, *args, **kw):
        self.events = []
        self.append = self.events.append
        html.parser.HTMLParser.__init__(self, *args, **kw)

    def get_events(self):
        L = []
        prevtype = None
        for event in self.events:
            type = event[0]
            if type == prevtype == 'data':
                L[-1] = ('data', L[-1][1] + event[1])
            else:
                L.append(event)
            prevtype = type
        self.events = L
        return L

    def handle_starttag(self, tag, attrs):
        self.append(('starttag', tag, attrs))

    def handle_startendtag(self, tag, attrs):
        self.append(('startendtag', tag, attrs))

    def handle_endtag(self, tag):
        self.append(('endtag', tag))

    def handle_comment(self, data):
        self.append(('comment', data))

    def handle_charref(self, data):
        self.append(('charref', data))

    def handle_data(self, data):
        self.append(('data', data))

    def handle_decl(self, data):
        self.append(('decl', data))

    def handle_entityref(self, data):
        self.append(('entityref', data))

    def handle_pi(self, data):
        self.append(('pi', data))

    def unknown_decl(self, decl):
        self.append(('unknown decl', decl))

class EventCollectorExtra(EventCollector):

    def handle_starttag(self, tag, attrs):
        EventCollector.handle_starttag(self, tag, attrs)
        self.append(('starttag_text', self.get_starttag_text()))

class EventCollectorCharrefs(EventCollector):

    def handle_charref(self, data):
        self.fail('This should never be called with convert_charrefs=True')

    def handle_entityref(self, data):
        self.fail('This should never be called with convert_charrefs=True')


# --- test body ---
def _run_check(source, expected_events, collector=None):
    if collector is None:
        collector = get_collector()
    parser = collector
    for s in source:
        parser.feed(s)
    parser.close()
    events = parser.get_events()
    if events != expected_events:

        raise AssertionError('received events did not match expected events' + '\nSource:\n' + repr(source) + '\nExpected:\n' + pprint.pformat(expected_events) + '\nReceived:\n' + pprint.pformat(events))

def _run_check_extra(source, events):
    _run_check(source, events, EventCollectorExtra(convert_charrefs=False))

def get_collector():
    return EventCollector(convert_charrefs=False)
_run_check('<a $><b $=%><c \\=/>', [('starttag', 'a', [('$', None)]), ('starttag', 'b', [('$', '%')]), ('starttag', 'c', [('\\', '/')])])
print("AttributesTestCase::test_attr_funky_names2: ok")
