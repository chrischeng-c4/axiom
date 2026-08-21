# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "perf"
# lib = "mako"
# dimension = "perf"
# case = "mako"
# subject = "pyperformance mako"
# kind = "bench"
# xfail = "mamba must run the pyperformance mako workload faster than CPython on CPU+RSS"
# mem_carveout = ""
# source = "pyperformance/data-files/benchmarks/bm_mako/run_benchmark.py"
# status = "filled"
# ///
# mamba-xfail: mamba must run the pyperformance mako workload faster than CPython on CPU+RSS
import sys as _sys, types as _t
class _Args:
    """Minimal argparser stand-in (no `import argparse`, which a sibling
    perf/argparse.py fixture would shadow). Records add_argument defaults."""
    def __init__(self):
        self._defaults = {}
    def add_argument(self, *names, **k):
        dest = k.get("dest")
        if not dest:
            for n in names:
                if isinstance(n, str) and n.startswith("--"):
                    dest = n[2:].replace("-", "_"); break
                if isinstance(n, str) and not n.startswith("-"):
                    dest = n; break
        if dest:
            self._defaults[dest] = k.get("default")
    def add_mutually_exclusive_group(self, *a, **k):
        return self
    def add_argument_group(self, *a, **k):
        return self
class _Runner:
    def __init__(self, *a, **k):
        self.metadata = {}
        self.argparser = _Args()
    def parse_args(self, *a, **k):
        return _t.SimpleNamespace(**self.argparser._defaults)
    def bench_func(self, name, func, *args, **k):
        func(*args)                       # func runs the workload itself
    def bench_time_func(self, name, func, *args, **k):
        func(1, *args)                    # pyperf passes loops as the 1st arg
    def bench_async_func(self, name, func, *args, **k):
        import asyncio
        asyncio.run(func(*args))
def _reg(_name, _code):
    _m = _t.ModuleType(_name)
    exec(compile(_code, _name, "exec"), _m.__dict__)
    _sys.modules[_name] = _m
_p = _t.ModuleType("pyperf")
_p.Runner = _Runner
def _pc():
    import time
    return time.perf_counter()
_p.perf_counter = _pc
_sys.modules["pyperf"] = _p

"""
Benchmark for test the performance of Mako templates engine.
Includes:
    -two template inherences
    -HTML escaping, XML escaping, URL escaping, whitespace trimming
    -function defitions and calls
    -forloops
"""

import functools
import sys

import pyperf

# Mako imports (w/o markupsafe)
sys.modules['markupsafe'] = None

import mako   # noqa
from mako.template import Template   # noqa
from mako.lookup import TemplateLookup   # noqa


__author__ = "virhilo@gmail.com (Lukasz Fidosz)"

LOREM_IPSUM = """Quisque lobortis hendrerit posuere. Curabitur
aliquet consequat sapien molestie pretium. Nunc adipiscing luc
tus mi, viverra porttitor lorem vulputate et. Ut at purus sem,
sed tincidunt ante. Vestibulum ante ipsum primis in faucibus
orci luctus et ultrices posuere cubilia Curae; Praesent pulvinar
sodales justo at congue. Praesent aliquet facilisis nisl a
molestie. Sed tempus nisl ut augue eleifend tincidunt. Sed a
lacinia nulla. Cras tortor est, mollis et consequat at,
vulputate et orci. Nulla sollicitudin"""

BASE_TEMPLATE = """
<%def name="render_table(table)">
    <table>
    % for row in table:
        <tr>
        % for col in row:
            <td>${col|h}</td>
        % endfor
        </tr>
    % endfor
    </table>
</%def>
<%def name="img(src, alt)">
    <img src="${src|u}" alt="${alt}" />
</%def>
<html>
    <head><title>${title|h,trim}</title></head>
    <body>
        ${next.body()}
    </body>
<html>
"""

PAGE_TEMPLATE = """
<%inherit file="base.mako"/>
<table>
    % for row in table:
        <tr>
            % for col in row:
                <td>${col}</td>
            % endfor
        </tr>
    % endfor
</table>
% for nr in range(img_count):
    ${parent.img('/foo/bar/baz.png', 'no image :o')}
% endfor
${next.body()}
% for nr in paragraphs:
    <p>${lorem|x}</p>
% endfor
${parent.render_table(table)}
"""

CONTENT_TEMPLATE = """
<%inherit file="page.mako"/>
<%def name="fun1()">
    <span>fun1</span>
</%def>
<%def name="fun2()">
    <span>fun2</span>
</%def>
<%def name="fun3()">
    <span>foo3</span>
</%def>
<%def name="fun4()">
    <span>foo4</span>
</%def>
<%def name="fun5()">
    <span>foo5</span>
</%def>
<%def name="fun6()">
    <span>foo6</span>
</%def>
<p>Lorem ipsum dolor sit amet, consectetur adipiscing elit.
Nam laoreet justo in velit faucibus lobortis. Sed dictum sagittis
volutpat. Sed adipiscing vestibulum consequat. Nullam laoreet, ante
nec pretium varius, libero arcu porttitor orci, id cursus odio nibh
nec leo. Vestibulum dapibus pellentesque purus, sed bibendum tortor
laoreet id. Praesent quis sodales ipsum. Fusce ut ligula sed diam
pretium sagittis vel at ipsum. Nulla sagittis sem quam, et volutpat
velit. Fusce dapibus ligula quis lectus ultricies tempor. Pellente</p>
${fun1()}
${fun2()}
${fun3()}
${fun4()}
${fun5()}
${fun6()}
"""


def bench_mako(runner, table_size, nparagraph, img_count):
    lookup = TemplateLookup()
    lookup.put_string('base.mako', BASE_TEMPLATE)
    lookup.put_string('page.mako', PAGE_TEMPLATE)

    template = Template(CONTENT_TEMPLATE, lookup=lookup)

    table = [range(table_size) for i in range(table_size)]
    paragraphs = range(nparagraph)
    title = 'Hello world!'

    func = functools.partial(template.render,
                             table=table, paragraphs=paragraphs,
                             lorem=LOREM_IPSUM, title=title,
                             img_count=img_count, range=range)
    runner.bench_func('mako', func)


if __name__ == "__main__":
    runner = pyperf.Runner()
    runner.metadata['description'] = "Mako templates"
    runner.metadata['mako_version'] = mako.__version__

    table_size = 150
    nparagraph = 50
    img_count = 50
    bench_mako(runner, table_size, nparagraph, img_count)
