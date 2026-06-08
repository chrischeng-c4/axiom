# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "operator"
# dimension = "real_world"
# case = "sort_group_records_pipeline"
# subject = "operator.attrgetter"
# kind = "semantic"
# xfail = "the pipeline keys on operator.itemgetter which returns 0 under mamba (repo-memory project_mamba_operator_itemgetter_returns_zero)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""operator.attrgetter: a records-processing pipeline sorts namedtuple rows by attrgetter, groups them with itertools.groupby, and uses methodcaller to normalize a text field — the canonical operator.* application combo"""
import collections
import itertools
import operator

Employee = collections.namedtuple("Employee", ["dept", "name", "salary"])
staff = [
    Employee("Sales", "alice", 60000),
    Employee("Eng", "bob", 95000),
    Employee("Sales", "carol", 72000),
    Employee("Eng", "dave", 88000),
    Employee("Eng", "erin", 91000),
]

# Sort then group by department via attrgetter (the classic groupby pattern).
by_dept = operator.attrgetter("dept")
staff_sorted = sorted(staff, key=by_dept)
grouped = {dept: list(rows) for dept, rows in itertools.groupby(staff_sorted, key=by_dept)}
assert sorted(grouped) == ["Eng", "Sales"], grouped
assert len(grouped["Eng"]) == 3, grouped["Eng"]
assert len(grouped["Sales"]) == 2, grouped["Sales"]

# Highest earner per department via attrgetter sort key.
by_salary = operator.attrgetter("salary")
top_eng = max(grouped["Eng"], key=by_salary)
assert top_eng.name == "bob", top_eng

# methodcaller normalizes the display name (title-case) for a report row.
titlecase = operator.methodcaller("title")
report = [titlecase(e.name) for e in staff_sorted]
assert "Alice" in report and "Bob" in report, report

# itemgetter pulls the salary column out of plain (name, salary) tuples for an aggregate.
pairs = [(e.name, e.salary) for e in staff]
salary_of = operator.itemgetter(1)
total_payroll = sum(salary_of(p) for p in pairs)
assert total_payroll == 406000, total_payroll

print("sort_group_records_pipeline OK")
