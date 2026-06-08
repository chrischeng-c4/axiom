"""
Integration tests for PostgreSQL window functions.

Tests cover:
- ROW_NUMBER() OVER (ORDER BY ...)
- RANK() and DENSE_RANK() with PARTITION BY
- LAG() and LEAD() window functions
- FIRST_VALUE() and LAST_VALUE()
- Aggregate window functions (SUM, AVG) with PARTITION BY
- Window functions with frame specifications
- Window functions combined with WHERE and GROUP BY
- Multiple window functions in the same query
"""
from decimal import Decimal
from datetime import date
from cclab.titan import execute, insert_one, Table, Column
from cclab.titan.query import WindowSpec
from cclab.probe import expect, fixture, test
from tests.postgres.base import PostgresSuite
@fixture
async def sales_table():
    """
    Create and populate a sales table for window function testing.

    Table structure:
    - id: SERIAL PRIMARY KEY
    - salesperson: VARCHAR(100)
    - region: VARCHAR(50)
    - amount: DECIMAL(10, 2)
    - sale_date: DATE
    """
    await execute('\n        CREATE TABLE IF NOT EXISTS sales (\n            id SERIAL PRIMARY KEY,\n            salesperson VARCHAR(100) NOT NULL,\n            region VARCHAR(50) NOT NULL,\n            amount DECIMAL(10, 2) NOT NULL,\n            sale_date DATE NOT NULL\n        )\n        ')
    test_data = [{'salesperson': 'Alice', 'region': 'North', 'amount': 1000.0, 'sale_date': '2024-01-15'}, {'salesperson': 'Alice', 'region': 'North', 'amount': 1500.0, 'sale_date': '2024-02-10'}, {'salesperson': 'Alice', 'region': 'North', 'amount': 2000.0, 'sale_date': '2024-03-05'}, {'salesperson': 'Bob', 'region': 'North', 'amount': 800.0, 'sale_date': '2024-01-20'}, {'salesperson': 'Bob', 'region': 'North', 'amount': 1200.0, 'sale_date': '2024-02-15'}, {'salesperson': 'Bob', 'region': 'North', 'amount': 900.0, 'sale_date': '2024-03-10'}, {'salesperson': 'Charlie', 'region': 'South', 'amount': 2200.0, 'sale_date': '2024-01-10'}, {'salesperson': 'Charlie', 'region': 'South', 'amount': 1800.0, 'sale_date': '2024-02-05'}, {'salesperson': 'Charlie', 'region': 'South', 'amount': 2500.0, 'sale_date': '2024-03-01'}, {'salesperson': 'Diana', 'region': 'South', 'amount': 1100.0, 'sale_date': '2024-01-25'}, {'salesperson': 'Diana', 'region': 'South', 'amount': 1400.0, 'sale_date': '2024-02-20'}, {'salesperson': 'Diana', 'region': 'South', 'amount': 1600.0, 'sale_date': '2024-03-15'}, {'salesperson': 'Eve', 'region': 'East', 'amount': 1700.0, 'sale_date': '2024-01-12'}, {'salesperson': 'Eve', 'region': 'East', 'amount': 1900.0, 'sale_date': '2024-02-08'}, {'salesperson': 'Eve', 'region': 'East', 'amount': 2100.0, 'sale_date': '2024-03-03'}, {'salesperson': 'Frank', 'region': 'East', 'amount': 1300.0, 'sale_date': '2024-01-18'}, {'salesperson': 'Frank', 'region': 'East', 'amount': 1500.0, 'sale_date': '2024-02-12'}, {'salesperson': 'Frank', 'region': 'East', 'amount': 1700.0, 'sale_date': '2024-03-08'}]
    for data in test_data:
        await insert_one('sales', data)
    yield

class TestRowNumberWindow(PostgresSuite):
    """Test ROW_NUMBER() window function."""

    @test
    async def test_row_number_order_by_amount(self, sales_table):
        """Test ROW_NUMBER() with ORDER BY to rank sales by amount."""
        results = await execute('\n            SELECT\n                salesperson,\n                region,\n                amount,\n                ROW_NUMBER() OVER (ORDER BY amount DESC) as rank\n            FROM sales\n            ORDER BY amount DESC\n            LIMIT 5\n            ')
        expect(len(results)).to_equal(5)
        expect(results[0]['rank']).to_equal(1)
        expect(results[1]['rank']).to_equal(2)
        expect(results[2]['rank']).to_equal(3)
        expect(results[3]['rank']).to_equal(4)
        expect(results[4]['rank']).to_equal(5)
        expect(float(results[0]['amount'])).to_equal(2500.0)
        expect(results[0]['salesperson']).to_equal('Charlie')

    @test
    async def test_row_number_partition_by_region(self, sales_table):
        """Test ROW_NUMBER() with PARTITION BY region."""
        results = await execute('\n            SELECT\n                salesperson,\n                region,\n                amount,\n                ROW_NUMBER() OVER (PARTITION BY region ORDER BY amount DESC) as region_rank\n            FROM sales\n            ORDER BY region, region_rank\n            ')
        by_region = {}
        for row in results:
            region = row['region']
            if region not in by_region:
                by_region[region] = []
            by_region[region].append(row)
        for region, rows in by_region.items():
            for i, row in enumerate(rows, start=1):
                expect(row['region_rank']).to_equal(i)
        east_sales = by_region['East']
        expect(float(east_sales[0]['amount'])).to_equal(2100.0)
        expect(east_sales[0]['salesperson']).to_equal('Eve')

class TestRankDenseRankWindow(PostgresSuite):
    """Test RANK() and DENSE_RANK() window functions."""

    @test
    async def test_rank_with_partition(self, sales_table):
        """Test RANK() with PARTITION BY and ORDER BY."""
        results = await execute("\n            SELECT\n                salesperson,\n                region,\n                amount,\n                RANK() OVER (PARTITION BY region ORDER BY amount DESC) as sales_rank\n            FROM sales\n            WHERE region = 'North'\n            ORDER BY sales_rank\n            ")
        expect(len(results)).to_equal(6)
        alice_top = [r for r in results if r['salesperson'] == 'Alice' and r['sales_rank'] == 1]
        expect(len(alice_top)).to_equal(1)
        expect(float(alice_top[0]['amount'])).to_equal(2000.0)

    @test
    async def test_dense_rank_window(self, sales_table):
        """Test DENSE_RANK() window function."""
        results = await execute('\n            SELECT\n                salesperson,\n                region,\n                amount,\n                DENSE_RANK() OVER (ORDER BY amount DESC) as dense_rank\n            FROM sales\n            ORDER BY dense_rank\n            LIMIT 10\n            ')
        expect(len(results)).to_equal(10)
        ranks = [r['dense_rank'] for r in results]
        expect(ranks[0]).to_equal(1)
        for i in range(1, len(ranks)):
            expect(ranks[i]).to_be_less_than_or_equal(ranks[i - 1] + 1)

class TestLagLeadWindow(PostgresSuite):
    """Test LAG() and LEAD() window functions."""

    @test
    async def test_lag_previous_sale_amount(self, sales_table):
        """Test LAG() to get previous sale amount for each salesperson."""
        results = await execute("\n            SELECT\n                salesperson,\n                sale_date,\n                amount,\n                LAG(amount) OVER (PARTITION BY salesperson ORDER BY sale_date) as prev_amount\n            FROM sales\n            WHERE salesperson = 'Alice'\n            ORDER BY sale_date\n            ")
        expect(len(results)).to_equal(3)
        expect(results[0]['prev_amount']).to_be_none()
        expect(float(results[0]['amount'])).to_equal(1000.0)
        expect(results[1]['prev_amount']).to_not_be_none()
        expect(float(results[1]['prev_amount'])).to_equal(1000.0)
        expect(float(results[1]['amount'])).to_equal(1500.0)
        expect(results[2]['prev_amount']).to_not_be_none()
        expect(float(results[2]['prev_amount'])).to_equal(1500.0)
        expect(float(results[2]['amount'])).to_equal(2000.0)

    @test
    async def test_lead_next_sale_amount(self, sales_table):
        """Test LEAD() to get next sale amount."""
        results = await execute("\n            SELECT\n                salesperson,\n                sale_date,\n                amount,\n                LEAD(amount) OVER (PARTITION BY salesperson ORDER BY sale_date) as next_amount\n            FROM sales\n            WHERE salesperson = 'Bob'\n            ORDER BY sale_date\n            ")
        expect(len(results)).to_equal(3)
        expect(results[0]['next_amount']).to_not_be_none()
        expect(float(results[0]['next_amount'])).to_equal(1200.0)
        expect(results[1]['next_amount']).to_not_be_none()
        expect(float(results[1]['next_amount'])).to_equal(900.0)
        expect(results[2]['next_amount']).to_be_none()

    @test
    async def test_lag_with_offset_and_default(self, sales_table):
        """Test LAG() with custom offset and default value."""
        results = await execute("\n            SELECT\n                salesperson,\n                sale_date,\n                amount,\n                LAG(amount, 2, 0) OVER (PARTITION BY salesperson ORDER BY sale_date) as prev_prev_amount\n            FROM sales\n            WHERE salesperson = 'Charlie'\n            ORDER BY sale_date\n            ")
        expect(len(results)).to_equal(3)
        expect(float(results[0]['prev_prev_amount'])).to_equal(0.0)
        expect(float(results[1]['prev_prev_amount'])).to_equal(0.0)
        expect(float(results[2]['prev_prev_amount'])).to_equal(2200.0)

class TestFirstLastValueWindow(PostgresSuite):
    """Test FIRST_VALUE() and LAST_VALUE() window functions."""

    @test
    async def test_first_value_by_region(self, sales_table):
        """Test FIRST_VALUE() to get first sale in each region."""
        results = await execute("\n            SELECT\n                salesperson,\n                region,\n                sale_date,\n                amount,\n                FIRST_VALUE(amount) OVER (\n                    PARTITION BY region\n                    ORDER BY sale_date\n                    ROWS BETWEEN UNBOUNDED PRECEDING AND UNBOUNDED FOLLOWING\n                ) as first_sale_amount\n            FROM sales\n            WHERE region = 'South'\n            ORDER BY sale_date\n            ")
        expect(len(results)).to_equal(6)
        first_amounts = {float(r['first_sale_amount']) for r in results}
        expect(len(first_amounts)).to_equal(1)
        expect(2200.0).to_be_in(first_amounts)

    @test
    async def test_last_value_by_region(self, sales_table):
        """Test LAST_VALUE() to get last sale in each region."""
        results = await execute("\n            SELECT\n                salesperson,\n                region,\n                sale_date,\n                amount,\n                LAST_VALUE(amount) OVER (\n                    PARTITION BY region\n                    ORDER BY sale_date\n                    ROWS BETWEEN UNBOUNDED PRECEDING AND UNBOUNDED FOLLOWING\n                ) as last_sale_amount\n            FROM sales\n            WHERE region = 'East'\n            ORDER BY sale_date\n            ")
        expect(len(results)).to_equal(6)
        last_amounts = {float(r['last_sale_amount']) for r in results}
        expect(len(last_amounts)).to_equal(1)
        expect(1700.0).to_be_in(last_amounts)

class TestAggregateWindowFunctions(PostgresSuite):
    """Test aggregate functions used as window functions."""

    @test
    async def test_sum_over_partition(self, sales_table):
        """Test SUM() OVER (PARTITION BY ...) for running totals."""
        results = await execute("\n            SELECT\n                salesperson,\n                region,\n                amount,\n                SUM(amount) OVER (PARTITION BY region) as region_total\n            FROM sales\n            WHERE region = 'North'\n            ORDER BY salesperson, amount\n            ")
        expect(len(results)).to_equal(6)
        totals = {float(r['region_total']) for r in results}
        expect(len(totals)).to_equal(1)
        expect(7400.0).to_be_in(totals)

    @test
    async def test_avg_over_with_order(self, sales_table):
        """Test AVG() OVER with ORDER BY for moving average."""
        results = await execute("\n            SELECT\n                salesperson,\n                sale_date,\n                amount,\n                AVG(amount) OVER (\n                    PARTITION BY salesperson\n                    ORDER BY sale_date\n                    ROWS BETWEEN UNBOUNDED PRECEDING AND CURRENT ROW\n                ) as cumulative_avg\n            FROM sales\n            WHERE salesperson = 'Alice'\n            ORDER BY sale_date\n            ")
        expect(len(results)).to_equal(3)
        expect(float(results[0]['cumulative_avg'])).to_be_close_to(1000.0, rel=0.01)
        expect(float(results[1]['cumulative_avg'])).to_be_close_to(1250.0, rel=0.01)
        expect(float(results[2]['cumulative_avg'])).to_be_close_to(1500.0, rel=0.01)

    @test
    async def test_sum_with_frame_specification(self, sales_table):
        """Test SUM() OVER with custom frame specification."""
        results = await execute("\n            SELECT\n                salesperson,\n                sale_date,\n                amount,\n                SUM(amount) OVER (\n                    PARTITION BY salesperson\n                    ORDER BY sale_date\n                    ROWS BETWEEN 1 PRECEDING AND CURRENT ROW\n                ) as two_sale_total\n            FROM sales\n            WHERE salesperson = 'Bob'\n            ORDER BY sale_date\n            ")
        expect(len(results)).to_equal(3)
        expect(float(results[0]['two_sale_total'])).to_equal(800.0)
        expect(float(results[1]['two_sale_total'])).to_equal(2000.0)
        expect(float(results[2]['two_sale_total'])).to_equal(2100.0)

class TestWindowWithWhereAndGroupBy(PostgresSuite):
    """Test window functions combined with WHERE and GROUP BY."""

    @test
    async def test_window_with_where_condition(self, sales_table):
        """Test window function with WHERE clause filtering."""
        results = await execute('\n            SELECT\n                salesperson,\n                region,\n                amount,\n                ROW_NUMBER() OVER (ORDER BY amount DESC) as rank\n            FROM sales\n            WHERE amount > 1500\n            ORDER BY rank\n            ')
        expect(all((float(r['amount']) > 1500 for r in results))).to_be_true()
        for i, row in enumerate(results, start=1):
            expect(row['rank']).to_equal(i)

    @test
    async def test_window_with_having(self, sales_table):
        """Test window function in subquery with GROUP BY and HAVING."""
        results = await execute('\n            SELECT\n                region,\n                total_sales,\n                RANK() OVER (ORDER BY total_sales DESC) as region_rank\n            FROM (\n                SELECT\n                    region,\n                    SUM(amount) as total_sales\n                FROM sales\n                GROUP BY region\n                HAVING SUM(amount) > 5000\n            ) regional_totals\n            ORDER BY region_rank\n            ')
        expect(all((float(r['total_sales']) > 5000 for r in results))).to_be_true()
        for i, row in enumerate(results, start=1):
            expect(row['region_rank']).to_equal(i)

    @test
    async def test_window_aggregates_by_salesperson(self, sales_table):
        """Test combining GROUP BY with window functions."""
        results = await execute('\n            SELECT\n                salesperson,\n                region,\n                total_amount,\n                AVG(total_amount) OVER (PARTITION BY region) as region_avg\n            FROM (\n                SELECT\n                    salesperson,\n                    region,\n                    SUM(amount) as total_amount\n                FROM sales\n                GROUP BY salesperson, region\n            ) salesperson_totals\n            ORDER BY region, salesperson\n            ')
        by_region = {}
        for row in results:
            region = row['region']
            if region not in by_region:
                by_region[region] = []
            by_region[region].append(row)
        for region, rows in by_region.items():
            region_avgs = {float(r['region_avg']) for r in rows}
            expect(len(region_avgs)).to_equal(1)

class TestMultipleWindowFunctions(PostgresSuite):
    """Test multiple window functions in the same query."""

    @test
    async def test_multiple_window_functions_same_query(self, sales_table):
        """Test combining multiple window functions."""
        results = await execute("\n            SELECT\n                salesperson,\n                region,\n                amount,\n                sale_date,\n                ROW_NUMBER() OVER (PARTITION BY region ORDER BY amount DESC) as region_rank,\n                RANK() OVER (ORDER BY amount DESC) as overall_rank,\n                LAG(amount) OVER (PARTITION BY salesperson ORDER BY sale_date) as prev_amount,\n                SUM(amount) OVER (PARTITION BY region) as region_total\n            FROM sales\n            WHERE region IN ('North', 'South')\n            ORDER BY region, region_rank\n            ")
        expect(all(('region_rank' in r for r in results))).to_be_true()
        expect(all(('overall_rank' in r for r in results))).to_be_true()
        expect(all(('region_total' in r for r in results))).to_be_true()
        expect(len(results)).to_equal(12)
        by_region = {}
        for row in results:
            region = row['region']
            if region not in by_region:
                by_region[region] = []
            by_region[region].append(row)
        for region, rows in by_region.items():
            for i, row in enumerate(rows, start=1):
                expect(row['region_rank']).to_equal(i)

    @test
    async def test_multiple_partitions_and_orders(self, sales_table):
        """Test window functions with different PARTITION BY and ORDER BY."""
        results = await execute("\n            SELECT\n                salesperson,\n                region,\n                amount,\n                sale_date,\n                ROW_NUMBER() OVER (PARTITION BY salesperson ORDER BY sale_date) as sale_sequence,\n                DENSE_RANK() OVER (PARTITION BY region ORDER BY amount DESC) as region_amount_rank,\n                FIRST_VALUE(sale_date) OVER (\n                    PARTITION BY salesperson\n                    ORDER BY sale_date\n                    ROWS BETWEEN UNBOUNDED PRECEDING AND UNBOUNDED FOLLOWING\n                ) as first_sale_date,\n                AVG(amount) OVER (PARTITION BY region) as region_avg_amount\n            FROM sales\n            WHERE salesperson IN ('Alice', 'Bob', 'Charlie')\n            ORDER BY salesperson, sale_date\n            ")
        expect(len(results)).to_equal(9)
        by_salesperson = {}
        for row in results:
            sp = row['salesperson']
            if sp not in by_salesperson:
                by_salesperson[sp] = []
            by_salesperson[sp].append(row)
        for sp, rows in by_salesperson.items():
            for i, row in enumerate(rows, start=1):
                expect(row['sale_sequence']).to_equal(i)
                if i > 1:
                    expect(row['first_sale_date']).to_equal(rows[0]['first_sale_date'])

class TestWindowEdgeCases(PostgresSuite):
    """Test edge cases and special scenarios for window functions."""

    @test
    async def test_window_with_empty_partition(self, sales_table):
        """Test window function when filtering leaves some partitions empty."""
        results = await execute('\n            SELECT\n                salesperson,\n                region,\n                amount,\n                ROW_NUMBER() OVER (PARTITION BY region ORDER BY amount DESC) as rank\n            FROM sales\n            WHERE amount > 2000\n            ORDER BY region, rank\n            ')
        expect(len(results)).to_be_greater_than_or_equal(3)
        expect(all((float(r['amount']) > 2000 for r in results))).to_be_true()

    @test
    async def test_window_with_null_handling(self, sales_table):
        """Test window function with LAG default for NULL values."""
        await insert_one('sales', {'salesperson': 'Grace', 'region': 'West', 'amount': 3000.0, 'sale_date': '2024-01-05'})
        results = await execute("\n            SELECT\n                salesperson,\n                sale_date,\n                amount,\n                LAG(amount, 1, -1) OVER (PARTITION BY salesperson ORDER BY sale_date) as prev_amount\n            FROM sales\n            WHERE salesperson = 'Grace'\n            ORDER BY sale_date\n            ")
        expect(len(results)).to_equal(1)
        expect(float(results[0]['prev_amount'])).to_equal(-1.0)

    @test
    async def test_window_with_limit(self, sales_table):
        """Test window function with LIMIT clause."""
        results = await execute('\n            SELECT\n                salesperson,\n                region,\n                amount,\n                ROW_NUMBER() OVER (ORDER BY amount DESC) as overall_rank\n            FROM sales\n            ORDER BY overall_rank\n            LIMIT 3\n            ')
        expect(len(results)).to_equal(3)
        expect(results[0]['overall_rank']).to_equal(1)
        expect(results[1]['overall_rank']).to_equal(2)
        expect(results[2]['overall_rank']).to_equal(3)
        expect(float(results[0]['amount'])).to_be_greater_than_or_equal(float(results[1]['amount']))
        expect(float(results[1]['amount'])).to_be_greater_than_or_equal(float(results[2]['amount']))