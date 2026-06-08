# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "perf"
# lib = "django_template"
# dimension = "perf"
# case = "django_template"
# subject = "pyperformance django_template"
# kind = "bench"
# xfail = "mamba must run the pyperformance django_template workload faster than CPython on CPU+RSS"
# mem_carveout = ""
# source = "pyperformance/data-files/benchmarks/bm_django_template/run_benchmark.py"
# status = "filled"
# ///
# mamba-xfail: mamba must run the pyperformance django_template workload faster than CPython on CPU+RSS
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

"""Test the performance of the Django template system.

This will have Django generate a 150x150-cell HTML table.
"""

import pyperf

import django.conf
from django.template import Context, Template


# 2016-10-10: Python 3.6 takes 380 ms
DEFAULT_SIZE = 100


def bench_django_template(runner, size):
    template = Template("""<table>
{% for row in table %}
<tr>{% for col in row %}<td>{{ col|escape }}</td>{% endfor %}</tr>
{% endfor %}
</table>
    """)
    table = [range(size) for _ in range(size)]
    context = Context({"table": table})

    runner.bench_func('django_template', template.render, context)


def prepare_cmd(runner, cmd):
    cmd.append("--table-size=%s" % runner.args.table_size)


if __name__ == "__main__":
    django.conf.settings.configure(TEMPLATES=[{
        'BACKEND': 'django.template.backends.django.DjangoTemplates',
    }])
    django.setup()

    runner = pyperf.Runner()
    cmd = runner.argparser
    cmd.add_argument("--table-size",
                     type=int, default=DEFAULT_SIZE,
                     help="Size of the HTML table, height and width "
                          "(default: %s)" % DEFAULT_SIZE)

    args = runner.parse_args()
    runner.metadata['description'] = "Django template"
    runner.metadata['django_version'] = django.__version__
    runner.metadata['django_table_size'] = args.table_size

    bench_django_template(runner, args.table_size)
