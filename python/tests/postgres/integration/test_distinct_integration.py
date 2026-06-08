"""
Integration tests for DISTINCT and DISTINCT ON functionality.

Tests query_aggregate with distinct and distinct_on parameters using a real PostgreSQL database.
"""
from datetime import date
from cclab.titan import execute, insert_one, query_aggregate
from cclab.probe import expect, fixture, test
from tests.postgres.base import PostgresSuite
@fixture
async def employees_table():
    """Create and populate an employees table for distinct testing."""
    await execute('\n        CREATE TABLE IF NOT EXISTS employees (\n            id SERIAL PRIMARY KEY,\n            department VARCHAR(100) NOT NULL,\n            job_role VARCHAR(100) NOT NULL,\n            salary DECIMAL(10, 2) NOT NULL,\n            hire_date DATE NOT NULL\n        )\n        ')
    test_data = [{'department': 'Engineering', 'job_role': 'Developer', 'salary': 80000.0, 'hire_date': date(2023, 1, 15)}, {'department': 'Engineering', 'job_role': 'Developer', 'salary': 85000.0, 'hire_date': date(2023, 2, 20)}, {'department': 'Engineering', 'job_role': 'Senior Developer', 'salary': 120000.0, 'hire_date': date(2022, 6, 10)}, {'department': 'Engineering', 'job_role': 'Manager', 'salary': 130000.0, 'hire_date': date(2021, 3, 5)}, {'department': 'Sales', 'job_role': 'Sales Rep', 'salary': 60000.0, 'hire_date': date(2023, 4, 12)}, {'department': 'Sales', 'job_role': 'Sales Rep', 'salary': 62000.0, 'hire_date': date(2023, 5, 18)}, {'department': 'Sales', 'job_role': 'Manager', 'salary': 95000.0, 'hire_date': date(2022, 1, 20)}, {'department': 'Marketing', 'job_role': 'Specialist', 'salary': 70000.0, 'hire_date': date(2023, 3, 8)}, {'department': 'Marketing', 'job_role': 'Manager', 'salary': 100000.0, 'hire_date': date(2021, 9, 15)}, {'department': 'Engineering', 'job_role': 'Developer', 'salary': 82000.0, 'hire_date': date(2023, 6, 1)}, {'department': 'Sales', 'job_role': 'Sales Rep', 'salary': 61000.0, 'hire_date': date(2023, 7, 10)}]
    for data in test_data:
        await insert_one('employees', data)
    yield

class TestDistinctSingleColumn(PostgresSuite):
    """Test DISTINCT on single column."""

    @test
    async def test_distinct_department(self, employees_table):
        """Test DISTINCT selecting unique departments."""
        results = await query_aggregate('employees', [('count_column', 'department', 'dept_count')], group_by=['department'], having=None, where_conditions=None, order_by=[('department', 'asc')], limit=None, distinct=True)
        expect(len(results)).to_equal(3)
        departments = [r['department'] for r in results]
        expect(set(departments)).to_equal({'Engineering', 'Sales', 'Marketing'})

    @test
    async def test_distinct_role(self, employees_table):
        """Test DISTINCT selecting unique roles."""
        results = await query_aggregate('employees', [('count_column', 'job_role', 'role_count')], group_by=['job_role'], having=None, where_conditions=None, order_by=[('job_role', 'asc')], limit=None, distinct=True)
        expect(len(results)).to_equal(5)
        roles = [r['job_role'] for r in results]
        expect('Developer').to_be_in(roles)
        expect('Senior Developer').to_be_in(roles)
        expect('Manager').to_be_in(roles)
        expect('Sales Rep').to_be_in(roles)
        expect('Specialist').to_be_in(roles)

    @test
    async def test_distinct_count(self, employees_table):
        """Test COUNT with DISTINCT to count unique values."""
        results = await query_aggregate('employees', [('count_distinct', 'department', 'unique_depts')], group_by=None, having=None, where_conditions=None, order_by=None, limit=None)
        expect(len(results)).to_equal(1)
        expect(results[0]['unique_depts']).to_equal(3)

class TestDistinctMultipleColumns(PostgresSuite):
    """Test DISTINCT on multiple columns."""

    @test
    async def test_distinct_department_role(self, employees_table):
        """Test DISTINCT on department and role combination."""
        results = await query_aggregate('employees', [('count', None, 'count')], group_by=['department', 'job_role'], having=None, where_conditions=None, order_by=[('department', 'asc'), ('job_role', 'asc')], limit=None, distinct=True)
        expect(len(results)).to_equal(7)

    @test
    async def test_distinct_multiple_with_aggregates(self, employees_table):
        """Test DISTINCT with multiple columns and aggregate functions."""
        results = await query_aggregate('employees', [('count', None, 'employee_count'), ('avg', 'salary', 'avg_salary')], group_by=['department', 'job_role'], having=None, where_conditions=None, order_by=[('department', 'asc'), ('job_role', 'asc')], limit=None, distinct=True)
        expect(len(results)).to_equal(7)
        eng_devs = [r for r in results if r['department'] == 'Engineering' and r['job_role'] == 'Developer']
        expect(len(eng_devs)).to_equal(1)
        expect(eng_devs[0]['employee_count']).to_equal(3)
        expect(float(eng_devs[0]['avg_salary'])).to_be_close_to(82333.33, rel=0.01)

class TestDistinctOn(PostgresSuite):
    """Test PostgreSQL DISTINCT ON functionality."""

    @test
    async def test_distinct_on_department(self, employees_table):
        """Test DISTINCT ON to get first employee per department."""
        results = await query_aggregate('employees', [('min', 'salary', 'min_salary'), ('count', None, 'count')], group_by=['department'], having=None, where_conditions=None, order_by=[('department', 'asc')], limit=None, distinct_on=['department'])
        expect(len(results)).to_equal(3)
        departments = [r['department'] for r in results]
        expect(len(set(departments))).to_equal(3)

    @test
    async def test_distinct_on_with_ordering(self, employees_table):
        """Test DISTINCT ON with specific ordering to get highest salary per department."""
        results = await query_aggregate('employees', [('max', 'salary', 'max_salary'), ('count', None, 'count')], group_by=['department'], having=None, where_conditions=None, order_by=[('department', 'asc'), ('max_salary', 'desc')], limit=None, distinct_on=['department'])
        expect(len(results)).to_equal(3)
        dept_results = {r['department']: r for r in results}
        expect('Engineering').to_be_in(dept_results)
        expect('Sales').to_be_in(dept_results)
        expect('Marketing').to_be_in(dept_results)

    @test
    async def test_distinct_on_multiple_columns(self, employees_table):
        """Test DISTINCT ON with multiple columns."""
        results = await query_aggregate('employees', [('min', 'salary', 'min_salary'), ('count', None, 'count')], group_by=['department', 'job_role'], having=None, where_conditions=None, order_by=[('department', 'asc'), ('job_role', 'asc')], limit=None, distinct_on=['department', 'job_role'])
        expect(len(results)).to_equal(7)

class TestDistinctWithAggregates(PostgresSuite):
    """Test DISTINCT combined with aggregate functions."""

    @test
    async def test_distinct_with_count_and_sum(self, employees_table):
        """Test DISTINCT with COUNT and SUM aggregates."""
        results = await query_aggregate('employees', [('count', None, 'emp_count'), ('sum', 'salary', 'total_salary')], group_by=['department'], having=None, where_conditions=None, order_by=[('department', 'asc')], limit=None, distinct=True)
        expect(len(results)).to_equal(3)
        eng_result = [r for r in results if r['department'] == 'Engineering']
        expect(len(eng_result)).to_equal(1)
        expect(eng_result[0]['emp_count']).to_equal(5)
        expect(float(eng_result[0]['total_salary'])).to_be_close_to(497000.0, rel=0.01)

    @test
    async def test_distinct_with_avg_and_min_max(self, employees_table):
        """Test DISTINCT with AVG, MIN, MAX aggregates."""
        results = await query_aggregate('employees', [('avg', 'salary', 'avg_salary'), ('min', 'salary', 'min_salary'), ('max', 'salary', 'max_salary')], group_by=['department'], having=None, where_conditions=None, order_by=[('department', 'asc')], limit=None, distinct=True)
        expect(len(results)).to_equal(3)
        sales_result = [r for r in results if r['department'] == 'Sales']
        expect(len(sales_result)).to_equal(1)
        expect(float(sales_result[0]['avg_salary'])).to_be_close_to(69500.0, rel=0.01)
        expect(float(sales_result[0]['min_salary'])).to_be_close_to(60000.0, rel=0.01)
        expect(float(sales_result[0]['max_salary'])).to_be_close_to(95000.0, rel=0.01)

    @test
    async def test_distinct_count_by_role(self, employees_table):
        """Test DISTINCT counting employees per role."""
        results = await query_aggregate('employees', [('count_distinct', 'department', 'dept_count')], group_by=['job_role'], having=None, where_conditions=None, order_by=[('job_role', 'asc')], limit=None)
        manager_result = [r for r in results if r['job_role'] == 'Manager']
        expect(len(manager_result)).to_equal(1)
        expect(manager_result[0]['dept_count']).to_equal(3)

class TestDistinctWithWhere(PostgresSuite):
    """Test DISTINCT combined with WHERE conditions."""

    @test
    async def test_distinct_with_salary_filter(self, employees_table):
        """Test DISTINCT with WHERE clause filtering by salary."""
        results = await query_aggregate('employees', [('count', None, 'high_earner_count')], group_by=['department'], having=None, where_conditions=[('salary', 'gte', 100000)], order_by=[('department', 'asc')], limit=None, distinct=True)
        expect(len(results)).to_equal(2)
        eng_result = [r for r in results if r['department'] == 'Engineering']
        expect(len(eng_result)).to_equal(1)
        expect(eng_result[0]['high_earner_count']).to_equal(2)
        mkt_result = [r for r in results if r['department'] == 'Marketing']
        expect(len(mkt_result)).to_equal(1)
        expect(mkt_result[0]['high_earner_count']).to_equal(1)

    @test
    async def test_distinct_with_role_filter(self, employees_table):
        """Test DISTINCT with WHERE filtering by role."""
        results = await query_aggregate('employees', [('count', None, 'count'), ('avg', 'salary', 'avg_salary')], group_by=['department'], having=None, where_conditions=[('job_role', 'eq', 'Manager')], order_by=[('department', 'asc')], limit=None, distinct=True)
        expect(len(results)).to_equal(3)
        for result in results:
            expect(result['count']).to_equal(1)

    @test
    async def test_distinct_with_multiple_conditions(self, employees_table):
        """Test DISTINCT with multiple WHERE conditions."""
        results = await query_aggregate('employees', [('count', None, 'count'), ('sum', 'salary', 'total')], group_by=['department'], having=None, where_conditions=[('salary', 'gt', 70000), ('job_role', 'ne', 'Manager')], order_by=[('department', 'asc')], limit=None, distinct=True)
        expect(len(results)).to_equal(1)
        expect(results[0]['department']).to_equal('Engineering')
        expect(results[0]['count']).to_equal(4)

    @test
    async def test_distinct_on_with_where(self, employees_table):
        """Test DISTINCT ON combined with WHERE clause."""
        results = await query_aggregate('employees', [('max', 'salary', 'max_salary'), ('count', None, 'count')], group_by=['department'], having=None, where_conditions=[('salary', 'lt', 100000)], order_by=[('department', 'asc')], limit=None, distinct_on=['department'])
        expect(len(results)).to_equal(3)

class TestDistinctEdgeCases(PostgresSuite):
    """Test edge cases and special scenarios."""

    @test
    async def test_distinct_with_limit(self, employees_table):
        """Test DISTINCT combined with LIMIT."""
        results = await query_aggregate('employees', [('count', None, 'count')], group_by=['department'], having=None, where_conditions=None, order_by=[('department', 'asc')], limit=2, distinct=True)
        expect(len(results)).to_equal(2)
        expect(results[0]['department']).to_equal('Engineering')
        expect(results[1]['department']).to_equal('Marketing')

    @test
    async def test_distinct_with_having(self, employees_table):
        """Test DISTINCT combined with HAVING clause."""
        results = await query_aggregate('employees', [('count', None, 'emp_count'), ('avg', 'salary', 'avg_salary')], group_by=['department'], having=[('count', None, 'gt', 2)], where_conditions=None, order_by=[('department', 'asc')], limit=None, distinct=True)
        expect(len(results)).to_equal(2)
        departments = [r['department'] for r in results]
        expect('Engineering').to_be_in(departments)
        expect('Sales').to_be_in(departments)
        expect('Marketing').to_not_be_in(departments)

    @test
    async def test_distinct_empty_result(self, employees_table):
        """Test DISTINCT with WHERE clause that returns no results."""
        results = await query_aggregate('employees', [('count', None, 'count')], group_by=['department'], having=None, where_conditions=[('salary', 'gt', 1000000)], order_by=None, limit=None, distinct=True)
        expect(len(results)).to_equal(0)

    @test
    async def test_distinct_all_columns_unique(self, employees_table):
        """Test DISTINCT when all rows are already unique."""
        results = await query_aggregate('employees', [('count', None, 'count')], group_by=['id'], having=None, where_conditions=None, order_by=[('id', 'asc')], limit=5, distinct=True)
        expect(len(results)).to_equal(5)
        for result in results:
            expect(result['count']).to_equal(1)