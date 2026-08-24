use super::super::super::super::harness::*;

/// Ported from `tests/cpython/behavior/std-libs/argparse/action_attribute_surface.py`.
#[test]
fn test_gen_behavior_std_libs_argparse_action_attribute_surface() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "argparse"
# dimension = "behavior"
# case = "action_attribute_surface"
# subject = "argparse.ArgumentParser.add_argument"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""argparse.ArgumentParser.add_argument: the Action returned by add_argument exposes every constructor keyword (nargs/const/default/type/choices/help/metavar/dest) as an attribute"""
import argparse

p = argparse.ArgumentParser()
act = p.add_argument(
    "--foo",
    nargs="?",
    const=42,
    default=84,
    type=int,
    choices=[1, 2],
    help="FOO",
    metavar="BAR",
    dest="baz",
)
assert act.nargs == "?", f"nargs = {act.nargs!r}"
assert act.const == 42, f"const = {act.const!r}"
assert act.default == 84, f"default = {act.default!r}"
assert act.type is int, f"type = {act.type!r}"
assert act.choices == [1, 2], f"choices = {act.choices!r}"
assert act.help == "FOO", f"help = {act.help!r}"
assert act.metavar == "BAR", f"metavar = {act.metavar!r}"
assert act.dest == "baz", f"dest = {act.dest!r}"
print("action_attribute_surface OK")
"###);
    assert_output(&out, r###"action_attribute_surface OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/argparse/append_collects_multiple.py`.
#[test]
fn test_gen_behavior_std_libs_argparse_append_collects_multiple() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "argparse"
# dimension = "behavior"
# case = "append_collects_multiple"
# subject = "argparse.ArgumentParser.add_argument"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""argparse.ArgumentParser.add_argument: action='append' collects repeated occurrences of the same option into an ordered list"""
import argparse

p = argparse.ArgumentParser()
p.add_argument("--item", action="append")
ns = p.parse_args(["--item", "a", "--item", "b", "--item", "c"])
assert ns.item == ["a", "b", "c"], f"append = {ns.item!r}"
print("append_collects_multiple OK")
"###);
    assert_output(&out, r###"append_collects_multiple OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/argparse/append_replaces_list_default.py`.
#[test]
fn test_gen_behavior_std_libs_argparse_append_replaces_list_default() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "argparse"
# dimension = "behavior"
# case = "append_replaces_list_default"
# subject = "argparse.ArgumentParser.add_argument"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""argparse.ArgumentParser.add_argument: with action='append' and a list default, supplied values replace (do not extend) the default, collecting only the parsed items"""
import argparse

p = argparse.ArgumentParser()
p.add_argument("--test", type=str, default=[], action="append")
ns = p.parse_args(["--test", "a", "--test", "b"])
assert ns.test == ["a", "b"], f"append over default = {ns.test!r}"
print("append_replaces_list_default OK")
"###);
    assert_output(&out, r###"append_replaces_list_default OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/argparse/custom_action_call.py`.
#[test]
fn test_gen_behavior_std_libs_argparse_custom_action_call() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "argparse"
# dimension = "behavior"
# case = "custom_action_call"
# subject = "argparse.Action"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""argparse.Action: a user-defined Action subclass has its __call__ invoked during parse_args, letting it write a derived value onto the Namespace"""
import argparse


class CollectAction(argparse.Action):
    def __call__(self, parser, namespace, values, option_string=None):
        setattr(namespace, self.dest, ["collected", values])


p = argparse.ArgumentParser()
p.add_argument("--spam", action=CollectAction)
ns = p.parse_args(["--spam", "eggs"])
assert ns.spam == ["collected", "eggs"], f"custom action result = {ns.spam!r}"
print("custom_action_call OK")
"###);
    assert_output(&out, r###"custom_action_call OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/argparse/custom_action_constructed_at_add_time.py`.
#[test]
fn test_gen_behavior_std_libs_argparse_custom_action_constructed_at_add_time() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "argparse"
# dimension = "behavior"
# case = "custom_action_constructed_at_add_time"
# subject = "argparse.Action"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""argparse.Action: a user-defined Action subclass is instantiated at add_argument time with the resolved constructor keywords (dest/const/default), before any parse"""
import argparse


class Probe(Exception):
    pass


class ProbingAction(argparse.Action):
    def __init__(self, option_strings, dest, const=None, default=None, **kwargs):
        if dest == "spam" and const == 99 and default == 7:
            raise Probe()
        super().__init__(option_strings, dest, **kwargs)

    def __call__(self, *args, **kwargs):
        pass


parser = argparse.ArgumentParser()
_raised = False
try:
    parser.add_argument("--spam", action=ProbingAction, const=99, default=7)
except Probe:
    _raised = True
assert _raised, "action class instantiated at add_argument time"
print("custom_action_constructed_at_add_time OK")
"###);
    assert_output(&out, r###"custom_action_constructed_at_add_time OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/argparse/dest_derivation.py`.
#[test]
fn test_gen_behavior_std_libs_argparse_dest_derivation() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "argparse"
# dimension = "behavior"
# case = "dest_derivation"
# subject = "argparse.ArgumentParser.add_argument"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""argparse.ArgumentParser.add_argument: dest is derived from the first long option, falling back to the first short option when no long option is given"""
import argparse

p = argparse.ArgumentParser()
assert p.add_argument("--foo").dest == "foo", "long-opt dest"
assert p.add_argument("-b", "--bar").dest == "bar", "long-opt wins over short"
assert p.add_argument("-x", "-y").dest == "x", "first short-opt dest"
print("dest_derivation OK")
"###);
    assert_output(&out, r###"dest_derivation OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/argparse/metavar_does_not_affect_parsing.py`.
#[test]
fn test_gen_behavior_std_libs_argparse_metavar_does_not_affect_parsing() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "argparse"
# dimension = "behavior"
# case = "metavar_does_not_affect_parsing"
# subject = "argparse.ArgumentParser.add_argument"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""argparse.ArgumentParser.add_argument: metavar= changes only the help display name; the default and the parsed value are unaffected"""
import argparse

p = argparse.ArgumentParser()
p.add_argument("--output", metavar="FILE", default="out.txt")
ns_default = p.parse_args([])
assert ns_default.output == "out.txt", f"metavar doesn't affect default = {ns_default.output!r}"
ns_parsed = p.parse_args(["--output", "result.csv"])
assert ns_parsed.output == "result.csv", f"metavar parsed = {ns_parsed.output!r}"
print("metavar_does_not_affect_parsing OK")
"###);
    assert_output(&out, r###"metavar_does_not_affect_parsing OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/argparse/namespace_attribute_access.py`.
#[test]
fn test_gen_behavior_std_libs_argparse_namespace_attribute_access() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "argparse"
# dimension = "behavior"
# case = "namespace_attribute_access"
# subject = "argparse.Namespace"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""argparse.Namespace: Namespace(**kwargs) turns keyword arguments into attributes readable via dot access and via vars() as a dict"""
import argparse

ns = argparse.Namespace(x=1, y="two")
assert ns.x == 1, f"Namespace.x = {ns.x!r}"
assert ns.y == "two", f"Namespace.y = {ns.y!r}"
d = vars(ns)
assert isinstance(d, dict), f"vars(Namespace) = {type(d)!r}"
assert d["x"] == 1, f"vars x = {d['x']!r}"
print("namespace_attribute_access OK")
"###);
    assert_output(&out, r###"namespace_attribute_access OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/argparse/namespace_eq_notimplemented_vs_other.py`.
#[test]
fn test_gen_behavior_std_libs_argparse_namespace_eq_notimplemented_vs_other() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "argparse"
# dimension = "behavior"
# case = "namespace_eq_notimplemented_vs_other"
# subject = "argparse.Namespace"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""argparse.Namespace: comparing a Namespace to a non-Namespace returns NotImplemented from __eq__/__ne__, so == None falls back to False and != None to True"""
import argparse

ns = argparse.Namespace(a=1, b=2)
assert ns.__eq__(None) is NotImplemented, "__eq__ vs None is NotImplemented"
assert ns.__ne__(None) is NotImplemented, "__ne__ vs None is NotImplemented"
assert (ns == None) is False, "== None falls back to False"
assert (ns != None) is True, "!= None falls back to True"
print("namespace_eq_notimplemented_vs_other OK")
"###);
    assert_output(&out, r###"namespace_eq_notimplemented_vs_other OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/argparse/namespace_equality_contents.py`.
#[test]
fn test_gen_behavior_std_libs_argparse_namespace_equality_contents() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "argparse"
# dimension = "behavior"
# case = "namespace_equality_contents"
# subject = "argparse.Namespace"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""argparse.Namespace: Namespace equality compares contents (order-independent); differing contents compare unequal"""
import argparse

ns1 = argparse.Namespace(a=1, b=2)
ns2 = argparse.Namespace(b=2, a=1)
ns3 = argparse.Namespace(a=1)
assert ns1 == ns2, "order-independent equality"
assert ns1 != ns3, "different contents unequal"
print("namespace_equality_contents OK")
"###);
    assert_output(&out, r###"namespace_equality_contents OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/argparse/namespace_membership.py`.
#[test]
fn test_gen_behavior_std_libs_argparse_namespace_membership() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "argparse"
# dimension = "behavior"
# case = "namespace_membership"
# subject = "argparse.Namespace"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""argparse.Namespace: the `in` operator tests attribute membership by name, independent of construction order, and reports absent names as not-in"""
import argparse

ns = argparse.Namespace(x=1, y=2)
assert "x" in ns, "x present"
assert "y" in ns, "y present"
assert "" not in ns, "empty string absent"
assert "xx" not in ns, "xx absent"
assert "z" not in ns, "z absent"
print("namespace_membership OK")
"###);
    assert_output(&out, r###"namespace_membership OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/argparse/namespace_missing_attr_raises.py`.
#[test]
fn test_gen_behavior_std_libs_argparse_namespace_missing_attr_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "argparse"
# dimension = "behavior"
# case = "namespace_missing_attr_raises"
# subject = "argparse.Namespace"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""argparse.Namespace: reading an unset attribute of an empty Namespace raises AttributeError"""
import argparse

ns = argparse.Namespace()
_raised = False
try:
    getattr(ns, "x")
except AttributeError:
    _raised = True
assert _raised, "missing attribute raises AttributeError"
print("namespace_missing_attr_raises OK")
"###);
    assert_output(&out, r###"namespace_missing_attr_raises OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/argparse/nargs_optional_const_default.py`.
#[test]
fn test_gen_behavior_std_libs_argparse_nargs_optional_const_default() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "argparse"
# dimension = "behavior"
# case = "nargs_optional_const_default"
# subject = "argparse.ArgumentParser.add_argument"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""argparse.ArgumentParser.add_argument: nargs='?' with const and default: absent option yields default, bare flag yields const, explicit value yields that value"""
import argparse

p = argparse.ArgumentParser()
p.add_argument("--verbose", nargs="?", const="C", default="D")
ns_absent = p.parse_args([])
assert ns_absent.verbose == "D", f"absent yields default = {ns_absent.verbose!r}"
ns_bare = p.parse_args(["--verbose"])
assert ns_bare.verbose == "C", f"bare flag yields const = {ns_bare.verbose!r}"
ns_value = p.parse_args(["--verbose", "V"])
assert ns_value.verbose == "V", f"explicit value = {ns_value.verbose!r}"
print("nargs_optional_const_default OK")
"###);
    assert_output(&out, r###"nargs_optional_const_default OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/argparse/nargs_star_zero_or_more.py`.
#[test]
fn test_gen_behavior_std_libs_argparse_nargs_star_zero_or_more() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "argparse"
# dimension = "behavior"
# case = "nargs_star_zero_or_more"
# subject = "argparse.ArgumentParser.add_argument"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""argparse.ArgumentParser.add_argument: nargs='*' yields an empty list for zero positionals and a list of all supplied positionals otherwise"""
import argparse

p = argparse.ArgumentParser()
p.add_argument("files", nargs="*")
ns_empty = p.parse_args([])
assert ns_empty.files == [], f"nargs=* empty = {ns_empty.files!r}"
ns_two = p.parse_args(["a.py", "b.py"])
assert ns_two.files == ["a.py", "b.py"], f"nargs=* two = {ns_two.files!r}"
print("nargs_star_zero_or_more OK")
"###);
    assert_output(&out, r###"nargs_star_zero_or_more OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/argparse/parse_args_accepts_tuple.py`.
#[test]
fn test_gen_behavior_std_libs_argparse_parse_args_accepts_tuple() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "argparse"
# dimension = "behavior"
# case = "parse_args_accepts_tuple"
# subject = "argparse.ArgumentParser.parse_args"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""argparse.ArgumentParser.parse_args: parse_args accepts a tuple argument vector, not just a list, parsing positionals and options identically"""
import argparse

p = argparse.ArgumentParser()
p.add_argument("x")
p.add_argument("--n", type=int, default=0)
ns = p.parse_args(("val", "--n", "7"))
assert ns.x == "val", f"tuple positional = {ns.x!r}"
assert ns.n == 7, f"tuple option = {ns.n!r}"
print("parse_args_accepts_tuple OK")
"###);
    assert_output(&out, r###"parse_args_accepts_tuple OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/argparse/parse_known_args_returns_extras.py`.
#[test]
fn test_gen_behavior_std_libs_argparse_parse_known_args_returns_extras() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "argparse"
# dimension = "behavior"
# case = "parse_known_args_returns_extras"
# subject = "argparse.ArgumentParser.parse_known_args"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""argparse.ArgumentParser.parse_known_args: parse_known_args returns (namespace, extras) where known options are parsed and unrecognized tokens land in the extras list"""
import argparse

p = argparse.ArgumentParser()
p.add_argument("--known", type=int, default=1)
ns, extras = p.parse_known_args(["--known", "2", "--unknown", "x"])
assert ns.known == 2, f"known = {ns.known!r}"
assert "--unknown" in extras, f"extras = {extras!r}"
print("parse_known_args_returns_extras OK")
"###);
    assert_output(&out, r###"parse_known_args_returns_extras OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/argparse/positional_argument.py`.
#[test]
fn test_gen_behavior_std_libs_argparse_positional_argument() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "argparse"
# dimension = "behavior"
# case = "positional_argument"
# subject = "argparse.ArgumentParser.add_argument"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""argparse.ArgumentParser.add_argument: a bare positional argument name binds the next argv token to that Namespace attribute"""
import argparse

p = argparse.ArgumentParser()
p.add_argument("name")
ns = p.parse_args(["hello"])
assert ns.name == "hello", f"positional = {ns.name!r}"
print("positional_argument OK")
"###);
    assert_output(&out, r###"positional_argument OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/argparse/set_defaults_overrides.py`.
#[test]
fn test_gen_behavior_std_libs_argparse_set_defaults_overrides() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "argparse"
# dimension = "behavior"
# case = "set_defaults_overrides"
# subject = "argparse.ArgumentParser.set_defaults"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""argparse.ArgumentParser.set_defaults: set_defaults overrides an add_argument default and injects extra Namespace attributes not declared as arguments"""
import argparse

p = argparse.ArgumentParser()
p.add_argument("--level", type=int, default=1)
p.set_defaults(level=5, extra="injected")
ns = p.parse_args([])
assert ns.level == 5, f"set_defaults overrides = {ns.level!r}"
assert ns.extra == "injected", f"set_defaults extra = {ns.extra!r}"
print("set_defaults_overrides OK")
"###);
    assert_output(&out, r###"set_defaults_overrides OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/argparse/store_true_false_flags.py`.
#[test]
fn test_gen_behavior_std_libs_argparse_store_true_false_flags() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "argparse"
# dimension = "behavior"
# case = "store_true_false_flags"
# subject = "argparse.ArgumentParser.add_argument"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""argparse.ArgumentParser.add_argument: action='store_true' defaults False and flips True when present; action='store_false' defaults True and flips False when present"""
import argparse

p = argparse.ArgumentParser()
p.add_argument("--debug", action="store_true")
p.add_argument("--quiet", action="store_false")
ns_default = p.parse_args([])
assert ns_default.debug == False, f"default debug = {ns_default.debug!r}"
assert ns_default.quiet == True, f"default quiet = {ns_default.quiet!r}"
ns_set = p.parse_args(["--debug", "--quiet"])
assert ns_set.debug == True, f"store_true = {ns_set.debug!r}"
assert ns_set.quiet == False, f"store_false = {ns_set.quiet!r}"
print("store_true_false_flags OK")
"###);
    assert_output(&out, r###"store_true_false_flags OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/argparse/subparsers_dispatch.py`.
#[test]
fn test_gen_behavior_std_libs_argparse_subparsers_dispatch() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "argparse"
# dimension = "behavior"
# case = "subparsers_dispatch"
# subject = "argparse.ArgumentParser.add_subparsers"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""argparse.ArgumentParser.add_subparsers: add_subparsers(dest=) records the chosen subcommand and the selected subparser's own options on the same Namespace"""
import argparse

p = argparse.ArgumentParser()
subs = p.add_subparsers(dest="cmd")
sub_run = subs.add_parser("run")
sub_run.add_argument("--fast", action="store_true")
ns = p.parse_args(["run", "--fast"])
assert ns.cmd == "run", f"subcommand = {ns.cmd!r}"
assert ns.fast == True, f"sub flag = {ns.fast!r}"
print("subparsers_dispatch OK")
"###);
    assert_output(&out, r###"subparsers_dispatch OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/argparse/type_coerces_value.py`.
#[test]
fn test_gen_behavior_std_libs_argparse_type_coerces_value() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "argparse"
# dimension = "behavior"
# case = "type_coerces_value"
# subject = "argparse.ArgumentParser.add_argument"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""argparse.ArgumentParser.add_argument: type= coerces the raw string into the declared type (float and str), so the Namespace attribute carries the converted value"""
import argparse

p = argparse.ArgumentParser()
p.add_argument("--num", type=float, default=0.0)
p.add_argument("--name", type=str, default="")
ns = p.parse_args(["--num", "3.14", "--name", "hello"])
assert isinstance(ns.num, float), f"float type = {type(ns.num)!r}"
assert abs(ns.num - 3.14) < 1e-9, f"float value = {ns.num!r}"
assert ns.name == "hello", f"str value = {ns.name!r}"
print("type_coerces_value OK")
"###);
    assert_output(&out, r###"type_coerces_value OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/argparse/type_not_applied_to_default.py`.
#[test]
fn test_gen_behavior_std_libs_argparse_type_not_applied_to_default() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "argparse"
# dimension = "behavior"
# case = "type_not_applied_to_default"
# subject = "argparse.ArgumentParser.add_argument"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""argparse.ArgumentParser.add_argument: type= is not applied to a default value (CPython bpo-15906): a list default with action='append' stays the untouched list when no value is supplied"""
import argparse

p = argparse.ArgumentParser()
p.add_argument("--test", type=str, default=[], action="append")
ns = p.parse_args([])
assert ns.test == [], f"type not applied to default = {ns.test!r}"
print("type_not_applied_to_default OK")
"###);
    assert_output(&out, r###"type_not_applied_to_default OK
"###);
}
