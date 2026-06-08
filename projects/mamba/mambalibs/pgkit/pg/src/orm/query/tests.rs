//! Tests for query builder module.

use super::*;
use crate::ExtractedValue;

    #[test]
    fn test_simple_select() {
        let qb = QueryBuilder::new("users").unwrap();
        let (sql, params) = qb.build_select();
        assert_eq!(sql, "SELECT * FROM \"users\"");
        assert_eq!(params.len(), 0);
    }

    #[test]
    fn test_select_with_columns() {
        let qb = QueryBuilder::new("users").unwrap()
            .select(vec!["id".to_string(), "name".to_string()]).unwrap();
        let (sql, params) = qb.build_select();
        assert_eq!(sql, "SELECT \"id\", \"name\" FROM \"users\"");
        assert_eq!(params.len(), 0);
    }

    #[test]
    fn test_select_with_where() {
        let qb = QueryBuilder::new("users").unwrap()
            .where_clause("id", Operator::Eq, ExtractedValue::Int(42)).unwrap();
        let (sql, params) = qb.build_select();
        assert_eq!(sql, "SELECT * FROM \"users\" WHERE \"id\" = $1");
        assert_eq!(params.len(), 1);
    }

    #[test]
    fn test_select_with_multiple_where() {
        let qb = QueryBuilder::new("users").unwrap()
            .where_clause("age", Operator::Gt, ExtractedValue::Int(18)).unwrap()
            .where_clause("status", Operator::Eq, ExtractedValue::String("active".to_string())).unwrap();
        let (sql, params) = qb.build_select();
        assert_eq!(sql, "SELECT * FROM \"users\" WHERE \"age\" > $1 AND \"status\" = $2");
        assert_eq!(params.len(), 2);
    }

    #[test]
    fn test_filter_by_builds_equality_conditions() {
        let qb = QueryBuilder::new("users").unwrap()
            .filter_by(&[
                ("active", ExtractedValue::Bool(true)),
                ("account_type", ExtractedValue::String("admin".to_string())),
            ]).unwrap();
        let (sql, params) = qb.build_select();
        assert_eq!(sql, "SELECT * FROM \"users\" WHERE \"active\" = $1 AND \"account_type\" = $2");
        assert_eq!(params, vec![
            ExtractedValue::Bool(true),
            ExtractedValue::String("admin".to_string()),
        ]);
    }

    #[test]
    fn test_filter_by_null_uses_is_null() {
        let qb = QueryBuilder::new("users").unwrap()
            .filter_by(&[
                ("deleted_at", ExtractedValue::Null),
                ("active", ExtractedValue::Bool(true)),
            ]).unwrap();
        let (sql, params) = qb.build_select();
        assert_eq!(sql, "SELECT * FROM \"users\" WHERE \"deleted_at\" IS NULL AND \"active\" = $1");
        assert_eq!(params, vec![ExtractedValue::Bool(true)]);
    }

    #[test]
    fn test_where_between_builds_two_bound_params() {
        let qb = QueryBuilder::new("orders").unwrap()
            .where_between("amount", ExtractedValue::Int(10), ExtractedValue::Int(20)).unwrap();
        let (sql, params) = qb.build_select();
        assert_eq!(sql, "SELECT * FROM \"orders\" WHERE \"amount\" BETWEEN $1 AND $2");
        assert_eq!(params, vec![ExtractedValue::Int(10), ExtractedValue::Int(20)]);
    }

    #[test]
    fn test_where_not_between_builds_two_bound_params() {
        let qb = QueryBuilder::new("events").unwrap()
            .where_not_between("created_day", ExtractedValue::Int(1), ExtractedValue::Int(7)).unwrap();
        let (sql, params) = qb.build_select();
        assert_eq!(sql, "SELECT * FROM \"events\" WHERE \"created_day\" NOT BETWEEN $1 AND $2");
        assert_eq!(params, vec![ExtractedValue::Int(1), ExtractedValue::Int(7)]);
    }

    #[test]
    fn test_query_builder_driver_fetch_methods_are_exposed() {
        let _fetch_rows = QueryBuilder::fetch_rows;
        let _fetch_optional_row = QueryBuilder::fetch_optional_row;
    }

    #[test]
    fn test_select_with_order_by() {
        let qb = QueryBuilder::new("users").unwrap()
            .order_by("created_at", OrderDirection::Desc).unwrap();
        let (sql, params) = qb.build_select();
        assert_eq!(sql, "SELECT * FROM \"users\" ORDER BY \"created_at\" DESC");
        assert_eq!(params.len(), 0);
    }

    #[test]
    fn test_select_with_limit_offset() {
        let qb = QueryBuilder::new("users").unwrap()
            .limit(10)
            .offset(20);
        let (sql, params) = qb.build_select();
        assert_eq!(sql, "SELECT * FROM \"users\" LIMIT $1 OFFSET $2");
        assert_eq!(params.len(), 2);
    }

    #[test]
    fn test_complex_select() {
        let qb = QueryBuilder::new("users").unwrap()
            .select(vec!["id".to_string(), "name".to_string(), "email".to_string()]).unwrap()
            .where_clause("age", Operator::Gte, ExtractedValue::Int(18)).unwrap()
            .where_clause("active", Operator::Eq, ExtractedValue::Bool(true)).unwrap()
            .order_by("name", OrderDirection::Asc).unwrap()
            .limit(50)
            .offset(100);
        let (sql, params) = qb.build_select();
        assert_eq!(
            sql,
            "SELECT \"id\", \"name\", \"email\" FROM \"users\" WHERE \"age\" >= $1 AND \"active\" = $2 ORDER BY \"name\" ASC LIMIT $3 OFFSET $4"
        );
        assert_eq!(params.len(), 4);
    }

    #[test]
    fn test_insert_query() {
        let qb = QueryBuilder::new("users").unwrap();
        let values = vec![
            ("name".to_string(), ExtractedValue::String("Alice".to_string())),
            ("age".to_string(), ExtractedValue::Int(30)),
        ];
        let (sql, params) = qb.build_insert(&values).unwrap();
        assert_eq!(sql, "INSERT INTO \"users\" (\"name\", \"age\") VALUES ($1, $2) RETURNING *");
        assert_eq!(params.len(), 2);
    }

    #[test]
    fn test_update_query() {
        let qb = QueryBuilder::new("users").unwrap()
            .where_clause("id", Operator::Eq, ExtractedValue::Int(42)).unwrap();
        let values = vec![
            ("name".to_string(), ExtractedValue::String("Bob".to_string())),
            ("age".to_string(), ExtractedValue::Int(35)),
        ];
        let (sql, params) = qb.build_update(&values).unwrap();
        assert_eq!(sql, "UPDATE \"users\" SET \"name\" = $1, \"age\" = $2 WHERE \"id\" = $3");
        assert_eq!(params.len(), 3);
    }

    #[test]
    fn test_delete_query() {
        let qb = QueryBuilder::new("users").unwrap()
            .where_clause("id", Operator::Eq, ExtractedValue::Int(42)).unwrap();
        let (sql, params) = qb.build_delete();
        assert_eq!(sql, "DELETE FROM \"users\" WHERE \"id\" = $1");
        assert_eq!(params.len(), 1);
    }

    #[test]
    fn test_where_is_null() {
        let qb = QueryBuilder::new("users").unwrap()
            .where_null("email").unwrap();
        let (sql, params) = qb.build_select();
        assert_eq!(sql, "SELECT * FROM \"users\" WHERE \"email\" IS NULL");
        assert_eq!(params.len(), 0);
    }

    #[test]
    fn test_where_is_not_null() {
        let qb = QueryBuilder::new("users").unwrap()
            .where_not_null("email").unwrap();
        let (sql, params) = qb.build_select();
        assert_eq!(sql, "SELECT * FROM \"users\" WHERE \"email\" IS NOT NULL");
        assert_eq!(params.len(), 0);
    }

    #[test]
    fn test_validate_identifier_valid() {
        assert!(QueryBuilder::validate_identifier("users").is_ok());
        assert!(QueryBuilder::validate_identifier("user_table").is_ok());
        assert!(QueryBuilder::validate_identifier("_private").is_ok());
        assert!(QueryBuilder::validate_identifier("table123").is_ok());
    }

    #[test]
    fn test_validate_identifier_invalid() {
        // Empty
        assert!(QueryBuilder::validate_identifier("").is_err());

        // Starts with number
        assert!(QueryBuilder::validate_identifier("123table").is_err());

        // Contains special characters
        assert!(QueryBuilder::validate_identifier("user-table").is_err());
        assert!(QueryBuilder::validate_identifier("user$table").is_err());

        // SQL keywords
        assert!(QueryBuilder::validate_identifier("select").is_err());
        assert!(QueryBuilder::validate_identifier("drop").is_err());
        assert!(QueryBuilder::validate_identifier("DELETE").is_err());

        // System catalogs
        assert!(QueryBuilder::validate_identifier("pg_catalog").is_err());
        assert!(QueryBuilder::validate_identifier("information_schema").is_err());

        // Too long (>63 characters)
        let long_name = "a".repeat(64);
        assert!(QueryBuilder::validate_identifier(&long_name).is_err());
    }

    #[test]
    fn test_validate_schema_qualified_identifiers() {
        // Valid schema-qualified names
        assert!(QueryBuilder::validate_identifier("public.users").is_ok());
        assert!(QueryBuilder::validate_identifier("public.bench_insert_one_db").is_ok());
        assert!(QueryBuilder::validate_identifier("myschema.mytable").is_ok());
        assert!(QueryBuilder::validate_identifier("_private._internal").is_ok());

        // Invalid: too many dots
        assert!(QueryBuilder::validate_identifier("schema.table.column").is_err());
        assert!(QueryBuilder::validate_identifier("a.b.c.d").is_err());

        // Invalid: empty parts
        assert!(QueryBuilder::validate_identifier(".table").is_err());
        assert!(QueryBuilder::validate_identifier("schema.").is_err());
        assert!(QueryBuilder::validate_identifier(".").is_err());

        // Invalid: system schema in qualified name
        assert!(QueryBuilder::validate_identifier("pg_catalog.users").is_err());
        assert!(QueryBuilder::validate_identifier("public.pg_internal").is_err());

        // Invalid: SQL keyword in qualified name
        assert!(QueryBuilder::validate_identifier("public.select").is_err());
        assert!(QueryBuilder::validate_identifier("drop.users").is_err());

        // Invalid: starts with number
        assert!(QueryBuilder::validate_identifier("public.123table").is_err());
        assert!(QueryBuilder::validate_identifier("123schema.table").is_err());

        // Invalid: special characters in parts
        assert!(QueryBuilder::validate_identifier("public.user-table").is_err());
        assert!(QueryBuilder::validate_identifier("my-schema.users").is_err());
    }

    #[test]
    fn test_new_with_invalid_table() {
        assert!(QueryBuilder::new("drop").is_err());
        assert!(QueryBuilder::new("pg_catalog").is_err());
        assert!(QueryBuilder::new("123table").is_err());
    }

    #[test]
    fn test_new_with_schema_qualified_table() {
        // Valid schema-qualified table names
        assert!(QueryBuilder::new("public.users").is_ok());
        assert!(QueryBuilder::new("public.bench_insert_one_db").is_ok());
        assert!(QueryBuilder::new("myschema.mytable").is_ok());

        // Test that queries work with schema-qualified names
        let qb = QueryBuilder::new("public.users").unwrap();
        let (sql, params) = qb.build_select();
        assert_eq!(sql, "SELECT * FROM \"public\".\"users\"");
        assert_eq!(params.len(), 0);

        // Test with WHERE clause
        let qb = QueryBuilder::new("public.bench_insert_one_db").unwrap()
            .where_clause("id", Operator::Eq, ExtractedValue::Int(1)).unwrap();
        let (sql, params) = qb.build_select();
        assert_eq!(sql, "SELECT * FROM \"public\".\"bench_insert_one_db\" WHERE \"id\" = $1");
        assert_eq!(params.len(), 1);
    }

    #[test]
    fn test_operators() {
        assert_eq!(Operator::Eq.to_sql(), "=");
        assert_eq!(Operator::Ne.to_sql(), "!=");
        assert_eq!(Operator::Gt.to_sql(), ">");
        assert_eq!(Operator::Gte.to_sql(), ">=");
        assert_eq!(Operator::Lt.to_sql(), "<");
        assert_eq!(Operator::Lte.to_sql(), "<=");
        assert_eq!(Operator::In.to_sql(), "IN");
        assert_eq!(Operator::NotIn.to_sql(), "NOT IN");
        assert_eq!(Operator::Between.to_sql(), "BETWEEN");
        assert_eq!(Operator::NotBetween.to_sql(), "NOT BETWEEN");
        assert_eq!(Operator::Like.to_sql(), "LIKE");
        assert_eq!(Operator::ILike.to_sql(), "ILIKE");
        assert_eq!(Operator::IsNull.to_sql(), "IS NULL");
        assert_eq!(Operator::IsNotNull.to_sql(), "IS NOT NULL");
        assert_eq!(Operator::InSubquery.to_sql(), "IN");
        assert_eq!(Operator::NotInSubquery.to_sql(), "NOT IN");
        assert_eq!(Operator::Exists.to_sql(), "EXISTS");
        assert_eq!(Operator::NotExists.to_sql(), "NOT EXISTS");
    }

    #[test]
    fn test_order_direction() {
        assert_eq!(OrderDirection::Asc.to_sql(), "ASC");
        assert_eq!(OrderDirection::Desc.to_sql(), "DESC");
    }

    #[test]
    fn test_like_operator() {
        let qb = QueryBuilder::new("users").unwrap()
            .where_clause("name", Operator::Like, ExtractedValue::String("%John%".to_string())).unwrap();
        let (sql, params) = qb.build_select();
        assert_eq!(sql, "SELECT * FROM \"users\" WHERE \"name\" LIKE $1");
        assert_eq!(params.len(), 1);
    }

    #[test]
    fn test_in_operator() {
        let qb = QueryBuilder::new("users").unwrap()
            .where_clause("status", Operator::In,
                ExtractedValue::Array(vec![
                    ExtractedValue::String("active".to_string()),
                    ExtractedValue::String("pending".to_string())
                ])
            ).unwrap();
        let (sql, params) = qb.build_select();
        assert_eq!(sql, "SELECT * FROM \"users\" WHERE \"status\" IN ($1)");
        assert_eq!(params.len(), 1);
    }

    #[test]
    fn test_empty_insert_fails() {
        let qb = QueryBuilder::new("users").unwrap();
        let values: Vec<(String, ExtractedValue)> = vec![];
        assert!(qb.build_insert(&values).is_err());
    }

    #[test]
    fn test_empty_update_fails() {
        let qb = QueryBuilder::new("users").unwrap();
        let values: Vec<(String, ExtractedValue)> = vec![];
        assert!(qb.build_update(&values).is_err());
    }

    #[test]
    fn test_insert_with_invalid_column_name() {
        let qb = QueryBuilder::new("users").unwrap();
        let values = vec![
            ("drop".to_string(), ExtractedValue::String("value".to_string())),
        ];
        assert!(qb.build_insert(&values).is_err());
    }

    #[test]
    fn test_update_with_invalid_column_name() {
        let qb = QueryBuilder::new("users").unwrap();
        let values = vec![
            ("select".to_string(), ExtractedValue::String("value".to_string())),
        ];
        assert!(qb.build_update(&values).is_err());
    }

    #[test]
    fn test_multiple_order_by() {
        let qb = QueryBuilder::new("users").unwrap()
            .order_by("created_at", OrderDirection::Desc).unwrap()
            .order_by("name", OrderDirection::Asc).unwrap();
        let (sql, params) = qb.build_select();
        assert_eq!(sql, "SELECT * FROM \"users\" ORDER BY \"created_at\" DESC, \"name\" ASC");
        assert_eq!(params.len(), 0);
    }

    #[test]
    fn test_complex_where_4_conditions() {
        let qb = QueryBuilder::new("users").unwrap()
            .where_clause("age", Operator::Gte, ExtractedValue::Int(18)).unwrap()
            .where_clause("status", Operator::Eq, ExtractedValue::String("active".to_string())).unwrap()
            .where_clause("score", Operator::Lt, ExtractedValue::Int(100)).unwrap()
            .where_clause("verified", Operator::Eq, ExtractedValue::Bool(true)).unwrap();
        let (sql, params) = qb.build_select();
        assert_eq!(
            sql,
            "SELECT * FROM \"users\" WHERE \"age\" >= $1 AND \"status\" = $2 AND \"score\" < $3 AND \"verified\" = $4"
        );
        assert_eq!(params.len(), 4);

        // Verify parameter values
        match &params[0] {
            ExtractedValue::Int(18) => {},
            _ => panic!("Expected Int(18)"),
        }
        match &params[1] {
            ExtractedValue::String(s) if s == "active" => {},
            _ => panic!("Expected String(active)"),
        }
        match &params[2] {
            ExtractedValue::Int(100) => {},
            _ => panic!("Expected Int(100)"),
        }
        match &params[3] {
            ExtractedValue::Bool(true) => {},
            _ => panic!("Expected Bool(true)"),
        }
    }

    #[test]
    fn test_schema_qualified_table_names() {
        // Test SELECT with schema-qualified table
        let qb = QueryBuilder::new("public.users").unwrap()
            .select(vec!["id".to_string(), "name".to_string()]).unwrap()
            .where_clause("active", Operator::Eq, ExtractedValue::Bool(true)).unwrap();
        let (sql, params) = qb.build_select();
        assert_eq!(sql, "SELECT \"id\", \"name\" FROM \"public\".\"users\" WHERE \"active\" = $1");
        assert_eq!(params.len(), 1);

        // Test INSERT with schema-qualified table
        let qb = QueryBuilder::new("myschema.products").unwrap();
        let values = vec![
            ("name".to_string(), ExtractedValue::String("Widget".to_string())),
            ("price".to_string(), ExtractedValue::Int(999)),
        ];
        let (sql, params) = qb.build_insert(&values).unwrap();
        assert_eq!(sql, "INSERT INTO \"myschema\".\"products\" (\"name\", \"price\") VALUES ($1, $2) RETURNING *");
        assert_eq!(params.len(), 2);

        // Test UPDATE with schema-qualified table
        let qb = QueryBuilder::new("analytics.events").unwrap()
            .where_clause("id", Operator::Eq, ExtractedValue::Int(42)).unwrap();
        let values = vec![
            ("processed".to_string(), ExtractedValue::Bool(true)),
        ];
        let (sql, params) = qb.build_update(&values).unwrap();
        assert_eq!(sql, "UPDATE \"analytics\".\"events\" SET \"processed\" = $1 WHERE \"id\" = $2");
        assert_eq!(params.len(), 2);

        // Test DELETE with schema-qualified table
        let qb = QueryBuilder::new("logs.audit_log").unwrap()
            .where_clause("created_at", Operator::Lt, ExtractedValue::String("2020-01-01".to_string())).unwrap();
        let (sql, params) = qb.build_delete();
        assert_eq!(sql, "DELETE FROM \"logs\".\"audit_log\" WHERE \"created_at\" < $1");
        assert_eq!(params.len(), 1);
    }

    #[test]
    fn test_very_large_limit() {
        // Test with i64::MAX
        let qb = QueryBuilder::new("users").unwrap()
            .limit(i64::MAX);
        let (sql, params) = qb.build_select();
        assert_eq!(sql, "SELECT * FROM \"users\" LIMIT $1");
        assert_eq!(params.len(), 1);
        match &params[0] {
            ExtractedValue::BigInt(val) if *val == i64::MAX => {},
            _ => panic!("Expected BigInt(i64::MAX)"),
        }

        // Test with a very large but reasonable limit
        let qb = QueryBuilder::new("users").unwrap()
            .limit(1_000_000_000);
        let (sql, params) = qb.build_select();
        assert_eq!(sql, "SELECT * FROM \"users\" LIMIT $1");
        assert_eq!(params.len(), 1);
        match &params[0] {
            ExtractedValue::BigInt(1_000_000_000) => {},
            _ => panic!("Expected BigInt(1_000_000_000)"),
        }
    }

    #[test]
    fn test_zero_limit() {
        // Test LIMIT 0 behavior - should be valid SQL
        let qb = QueryBuilder::new("users").unwrap()
            .limit(0);
        let (sql, params) = qb.build_select();
        assert_eq!(sql, "SELECT * FROM \"users\" LIMIT $1");
        assert_eq!(params.len(), 1);
        match &params[0] {
            ExtractedValue::BigInt(0) => {},
            _ => panic!("Expected BigInt(0)"),
        }

        // Test LIMIT 0 with WHERE clause
        let qb = QueryBuilder::new("users").unwrap()
            .where_clause("active", Operator::Eq, ExtractedValue::Bool(true)).unwrap()
            .limit(0);
        let (sql, params) = qb.build_select();
        assert_eq!(sql, "SELECT * FROM \"users\" WHERE \"active\" = $1 LIMIT $2");
        assert_eq!(params.len(), 2);

        // Test LIMIT 0 with ORDER BY
        let qb = QueryBuilder::new("users").unwrap()
            .order_by("created_at", OrderDirection::Desc).unwrap()
            .limit(0);
        let (sql, params) = qb.build_select();
        assert_eq!(sql, "SELECT * FROM \"users\" ORDER BY \"created_at\" DESC LIMIT $1");
        assert_eq!(params.len(), 1);

        // Test LIMIT 0 with OFFSET
        let qb = QueryBuilder::new("users").unwrap()
            .limit(0)
            .offset(10);
        let (sql, params) = qb.build_select();
        assert_eq!(sql, "SELECT * FROM \"users\" LIMIT $1 OFFSET $2");
        assert_eq!(params.len(), 2);
        match &params[0] {
            ExtractedValue::BigInt(0) => {},
            _ => panic!("Expected BigInt(0) for limit"),
        }
        match &params[1] {
            ExtractedValue::BigInt(10) => {},
            _ => panic!("Expected BigInt(10) for offset"),
        }
    }

    #[test]
    fn test_negative_offset() {
        // PostgreSQL allows negative offsets (they're treated as 0)
        // The query builder should accept them and let PostgreSQL handle it
        let qb = QueryBuilder::new("users").unwrap()
            .offset(-10);
        let (sql, params) = qb.build_select();
        assert_eq!(sql, "SELECT * FROM \"users\" OFFSET $1");
        assert_eq!(params.len(), 1);
        match &params[0] {
            ExtractedValue::BigInt(-10) => {},
            _ => panic!("Expected BigInt(-10)"),
        }

        // Test negative offset with positive limit
        let qb = QueryBuilder::new("users").unwrap()
            .limit(20)
            .offset(-5);
        let (sql, params) = qb.build_select();
        assert_eq!(sql, "SELECT * FROM \"users\" LIMIT $1 OFFSET $2");
        assert_eq!(params.len(), 2);
    }

    #[test]
    fn test_empty_select_columns() {
        // When no columns are specified, should default to SELECT *
        let qb = QueryBuilder::new("users").unwrap()
            .select(vec![]).unwrap();
        let (sql, params) = qb.build_select();
        assert_eq!(sql, "SELECT * FROM \"users\"");
        assert_eq!(params.len(), 0);

        // Empty select with WHERE clause
        let qb = QueryBuilder::new("users").unwrap()
            .select(vec![]).unwrap()
            .where_clause("active", Operator::Eq, ExtractedValue::Bool(true)).unwrap();
        let (sql, params) = qb.build_select();
        assert_eq!(sql, "SELECT * FROM \"users\" WHERE \"active\" = $1");
        assert_eq!(params.len(), 1);
    }

    #[test]
    fn test_in_operator_with_empty_list() {
        // IN operator with empty array
        let qb = QueryBuilder::new("users").unwrap()
            .where_clause("status", Operator::In, ExtractedValue::Array(vec![])).unwrap();
        let (sql, params) = qb.build_select();
        assert_eq!(sql, "SELECT * FROM \"users\" WHERE \"status\" IN ($1)");
        assert_eq!(params.len(), 1);

        // Verify the parameter is an empty array
        match &params[0] {
            ExtractedValue::Array(arr) if arr.is_empty() => {},
            _ => panic!("Expected empty Array"),
        }

        // NOT IN with empty array
        let qb = QueryBuilder::new("users").unwrap()
            .where_clause("user_role", Operator::NotIn, ExtractedValue::Array(vec![])).unwrap();
        let (sql, params) = qb.build_select();
        assert_eq!(sql, "SELECT * FROM \"users\" WHERE \"user_role\" NOT IN ($1)");
        assert_eq!(params.len(), 1);
    }

    #[test]
    fn test_in_operator_with_many_values() {
        // Create an array with 150 values
        let values: Vec<ExtractedValue> = (0..150)
            .map(|i| ExtractedValue::Int(i))
            .collect();

        let qb = QueryBuilder::new("users").unwrap()
            .where_clause("id", Operator::In, ExtractedValue::Array(values.clone())).unwrap();
        let (sql, params) = qb.build_select();
        assert_eq!(sql, "SELECT * FROM \"users\" WHERE \"id\" IN ($1)");
        assert_eq!(params.len(), 1);

        // Verify the parameter contains 150 values
        match &params[0] {
            ExtractedValue::Array(arr) if arr.len() == 150 => {},
            _ => panic!("Expected Array with 150 elements"),
        }

        // Test with strings
        let string_values: Vec<ExtractedValue> = (0..100)
            .map(|i| ExtractedValue::String(format!("value_{}", i)))
            .collect();

        let qb = QueryBuilder::new("users").unwrap()
            .where_clause("status", Operator::In, ExtractedValue::Array(string_values)).unwrap();
        let (sql, params) = qb.build_select();
        assert_eq!(sql, "SELECT * FROM \"users\" WHERE \"status\" IN ($1)");
        assert_eq!(params.len(), 1);

        match &params[0] {
            ExtractedValue::Array(arr) if arr.len() == 100 => {},
            _ => panic!("Expected Array with 100 elements"),
        }
    }

    #[test]
    fn test_like_patterns_escaping() {
        // Test basic LIKE patterns
        let qb = QueryBuilder::new("users").unwrap()
            .where_clause("name", Operator::Like, ExtractedValue::String("%John%".to_string())).unwrap();
        let (sql, params) = qb.build_select();
        assert_eq!(sql, "SELECT * FROM \"users\" WHERE \"name\" LIKE $1");
        assert_eq!(params.len(), 1);
        match &params[0] {
            ExtractedValue::String(s) if s == "%John%" => {},
            _ => panic!("Expected String(%John%)"),
        }

        // Test LIKE with special characters (underscore wildcard)
        let qb = QueryBuilder::new("users").unwrap()
            .where_clause("email", Operator::Like, ExtractedValue::String("user_@%.com".to_string())).unwrap();
        let (sql, params) = qb.build_select();
        assert_eq!(sql, "SELECT * FROM \"users\" WHERE \"email\" LIKE $1");
        match &params[0] {
            ExtractedValue::String(s) if s == "user_@%.com" => {},
            _ => panic!("Expected String(user_@%.com)"),
        }

        // Test ILIKE (case-insensitive)
        let qb = QueryBuilder::new("products").unwrap()
            .where_clause("description", Operator::ILike, ExtractedValue::String("%widget%".to_string())).unwrap();
        let (sql, params) = qb.build_select();
        assert_eq!(sql, "SELECT * FROM \"products\" WHERE \"description\" ILIKE $1");
        match &params[0] {
            ExtractedValue::String(s) if s == "%widget%" => {},
            _ => panic!("Expected String(%widget%)"),
        }

        // Test multiple LIKE conditions
        let qb = QueryBuilder::new("users").unwrap()
            .where_clause("first_name", Operator::Like, ExtractedValue::String("J%".to_string())).unwrap()
            .where_clause("last_name", Operator::Like, ExtractedValue::String("%son".to_string())).unwrap();
        let (sql, params) = qb.build_select();
        assert_eq!(sql, "SELECT * FROM \"users\" WHERE \"first_name\" LIKE $1 AND \"last_name\" LIKE $2");
        assert_eq!(params.len(), 2);
    }

    #[test]
    fn test_ilike_case_insensitive() {
        // Test ILIKE basic usage
        let qb = QueryBuilder::new("users").unwrap()
            .where_clause("name", Operator::ILike, ExtractedValue::String("%john%".to_string())).unwrap();
        let (sql, params) = qb.build_select();
        assert_eq!(sql, "SELECT * FROM \"users\" WHERE \"name\" ILIKE $1");
        assert_eq!(params.len(), 1);
        match &params[0] {
            ExtractedValue::String(s) if s == "%john%" => {},
            _ => panic!("Expected String(%john%)"),
        }

        // Test ILIKE with mixed case pattern
        let qb = QueryBuilder::new("products").unwrap()
            .where_clause("name", Operator::ILike, ExtractedValue::String("%WiDgEt%".to_string())).unwrap();
        let (sql, params) = qb.build_select();
        assert_eq!(sql, "SELECT * FROM \"products\" WHERE \"name\" ILIKE $1");
        match &params[0] {
            ExtractedValue::String(s) if s == "%WiDgEt%" => {},
            _ => panic!("Expected String(%WiDgEt%)"),
        }

        // Test ILIKE with underscore wildcard
        let qb = QueryBuilder::new("emails").unwrap()
            .where_clause("address", Operator::ILike, ExtractedValue::String("USER_@example.com".to_string())).unwrap();
        let (sql, params) = qb.build_select();
        assert_eq!(sql, "SELECT * FROM \"emails\" WHERE \"address\" ILIKE $1");
        match &params[0] {
            ExtractedValue::String(s) if s == "USER_@example.com" => {},
            _ => panic!("Expected String(USER_@example.com)"),
        }

        // Test combining ILIKE with other conditions
        let qb = QueryBuilder::new("articles").unwrap()
            .where_clause("title", Operator::ILike, ExtractedValue::String("%RUST%".to_string())).unwrap()
            .where_clause("published", Operator::Eq, ExtractedValue::Bool(true)).unwrap()
            .order_by("created_at", OrderDirection::Desc).unwrap()
            .limit(10);
        let (sql, params) = qb.build_select();
        assert_eq!(
            sql,
            "SELECT * FROM \"articles\" WHERE \"title\" ILIKE $1 AND \"published\" = $2 ORDER BY \"created_at\" DESC LIMIT $3"
        );
        assert_eq!(params.len(), 3);

        // Test multiple ILIKE conditions
        let qb = QueryBuilder::new("users").unwrap()
            .where_clause("first_name", Operator::ILike, ExtractedValue::String("j%".to_string())).unwrap()
            .where_clause("last_name", Operator::ILike, ExtractedValue::String("%SON".to_string())).unwrap();
        let (sql, params) = qb.build_select();
        assert_eq!(sql, "SELECT * FROM \"users\" WHERE \"first_name\" ILIKE $1 AND \"last_name\" ILIKE $2");
        assert_eq!(params.len(), 2);
    }

    #[test]
    fn test_order_by_multiple_columns() {
        // Test ORDER BY with 3 columns
        let qb = QueryBuilder::new("users").unwrap()
            .order_by("department", OrderDirection::Asc).unwrap()
            .order_by("salary", OrderDirection::Desc).unwrap()
            .order_by("name", OrderDirection::Asc).unwrap();
        let (sql, params) = qb.build_select();
        assert_eq!(sql, "SELECT * FROM \"users\" ORDER BY \"department\" ASC, \"salary\" DESC, \"name\" ASC");
        assert_eq!(params.len(), 0);

        // Test with 5 columns
        let qb = QueryBuilder::new("products").unwrap()
            .order_by("category", OrderDirection::Asc).unwrap()
            .order_by("subcategory", OrderDirection::Asc).unwrap()
            .order_by("price", OrderDirection::Desc).unwrap()
            .order_by("rating", OrderDirection::Desc).unwrap()
            .order_by("name", OrderDirection::Asc).unwrap();
        let (sql, params) = qb.build_select();
        assert_eq!(
            sql,
            "SELECT * FROM \"products\" ORDER BY \"category\" ASC, \"subcategory\" ASC, \"price\" DESC, \"rating\" DESC, \"name\" ASC"
        );
        assert_eq!(params.len(), 0);

        // Test ORDER BY with WHERE clause
        let qb = QueryBuilder::new("users").unwrap()
            .where_clause("active", Operator::Eq, ExtractedValue::Bool(true)).unwrap()
            .order_by("created_at", OrderDirection::Desc).unwrap()
            .order_by("id", OrderDirection::Asc).unwrap();
        let (sql, params) = qb.build_select();
        assert_eq!(sql, "SELECT * FROM \"users\" WHERE \"active\" = $1 ORDER BY \"created_at\" DESC, \"id\" ASC");
        assert_eq!(params.len(), 1);
    }

    #[test]
    fn test_combined_query_builder() {
        // Test combining all query options: SELECT columns, WHERE (multiple conditions),
        // ORDER BY (multiple), LIMIT, and OFFSET
        let qb = QueryBuilder::new("orders").unwrap()
            .select(vec![
                "id".to_string(),
                "customer_id".to_string(),
                "total".to_string(),
                "status".to_string(),
                "created_at".to_string(),
            ]).unwrap()
            .where_clause("status", Operator::In, ExtractedValue::Array(vec![
                ExtractedValue::String("pending".to_string()),
                ExtractedValue::String("processing".to_string()),
                ExtractedValue::String("shipped".to_string()),
            ])).unwrap()
            .where_clause("total", Operator::Gte, ExtractedValue::Int(100)).unwrap()
            .where_clause("customer_id", Operator::Ne, ExtractedValue::Int(0)).unwrap()
            .where_not_null("payment_method").unwrap()
            .order_by("created_at", OrderDirection::Desc).unwrap()
            .order_by("total", OrderDirection::Desc).unwrap()
            .order_by("id", OrderDirection::Asc).unwrap()
            .limit(50)
            .offset(100);

        let (sql, params) = qb.build_select();
        assert_eq!(
            sql,
            "SELECT \"id\", \"customer_id\", \"total\", \"status\", \"created_at\" FROM \"orders\" WHERE \"status\" IN ($1) AND \"total\" >= $2 AND \"customer_id\" != $3 AND \"payment_method\" IS NOT NULL ORDER BY \"created_at\" DESC, \"total\" DESC, \"id\" ASC LIMIT $4 OFFSET $5"
        );
        assert_eq!(params.len(), 5);

        // Verify parameter types
        match &params[0] {
            ExtractedValue::Array(arr) if arr.len() == 3 => {},
            _ => panic!("Expected Array with 3 elements"),
        }
        match &params[1] {
            ExtractedValue::Int(100) => {},
            _ => panic!("Expected Int(100)"),
        }
        match &params[2] {
            ExtractedValue::Int(0) => {},
            _ => panic!("Expected Int(0)"),
        }
        match &params[3] {
            ExtractedValue::BigInt(50) => {},
            _ => panic!("Expected BigInt(50)"),
        }
        match &params[4] {
            ExtractedValue::BigInt(100) => {},
            _ => panic!("Expected BigInt(100)"),
        }

        // Test combined query with schema-qualified table
        let qb = QueryBuilder::new("public.analytics_events").unwrap()
            .select(vec!["event_type".to_string(), "user_id".to_string(), "timestamp".to_string()]).unwrap()
            .where_clause("event_type", Operator::Like, ExtractedValue::String("click%".to_string())).unwrap()
            .where_clause("timestamp", Operator::Gte, ExtractedValue::String("2024-01-01".to_string())).unwrap()
            .order_by("timestamp", OrderDirection::Desc).unwrap()
            .limit(1000);

        let (sql, params) = qb.build_select();
        assert_eq!(
            sql,
            "SELECT \"event_type\", \"user_id\", \"timestamp\" FROM \"public\".\"analytics_events\" WHERE \"event_type\" LIKE $1 AND \"timestamp\" >= $2 ORDER BY \"timestamp\" DESC LIMIT $3"
        );
        assert_eq!(params.len(), 3);
    }

    #[test]
    fn test_upsert_single_conflict() {
        let qb = QueryBuilder::new("users").unwrap();
        let values = vec![
            ("email".to_string(), ExtractedValue::String("alice@example.com".to_string())),
            ("name".to_string(), ExtractedValue::String("Alice".to_string())),
            ("age".to_string(), ExtractedValue::Int(30)),
        ];
        let conflict_target = vec!["email".to_string()];
        let (sql, params) = qb.build_upsert(&values, &conflict_target, None).unwrap();

        assert!(sql.contains("INSERT INTO \"users\""));
        assert!(sql.contains("ON CONFLICT (\"email\")"));
        assert!(sql.contains("DO UPDATE SET"));
        assert!(sql.contains("\"name\" = EXCLUDED.\"name\""));
        assert!(sql.contains("\"age\" = EXCLUDED.\"age\""));
        assert!(sql.contains("RETURNING *"));
        assert_eq!(params.len(), 3);
    }

    #[test]
    fn test_upsert_selective_update() {
        let qb = QueryBuilder::new("users").unwrap();
        let values = vec![
            ("email".to_string(), ExtractedValue::String("alice@example.com".to_string())),
            ("name".to_string(), ExtractedValue::String("Alice".to_string())),
            ("age".to_string(), ExtractedValue::Int(30)),
        ];
        let conflict_target = vec!["email".to_string()];
        let update_cols = vec!["name".to_string()];
        let (sql, params) = qb.build_upsert(&values, &conflict_target, Some(&update_cols)).unwrap();

        assert!(sql.contains("DO UPDATE SET \"name\" = EXCLUDED.\"name\""));
        assert!(!sql.contains("\"age\" = EXCLUDED.\"age\""));
        assert_eq!(params.len(), 3);
    }

    #[test]
    fn test_upsert_composite_key() {
        let qb = QueryBuilder::new("users").unwrap();
        let values = vec![
            ("email".to_string(), ExtractedValue::String("alice@example.com".to_string())),
            ("department".to_string(), ExtractedValue::String("Engineering".to_string())),
            ("name".to_string(), ExtractedValue::String("Alice".to_string())),
        ];
        let conflict_target = vec!["email".to_string(), "department".to_string()];
        let (sql, params) = qb.build_upsert(&values, &conflict_target, None).unwrap();

        assert!(sql.contains("ON CONFLICT (\"email\", \"department\")"));
        assert!(sql.contains("DO UPDATE SET \"name\" = EXCLUDED.\"name\""));
        assert_eq!(params.len(), 3);
    }

    #[test]
    fn test_upsert_empty_conflict_target() {
        let qb = QueryBuilder::new("users").unwrap();
        let values = vec![
            ("email".to_string(), ExtractedValue::String("alice@example.com".to_string())),
        ];
        let conflict_target: Vec<String> = vec![];
        let result = qb.build_upsert(&values, &conflict_target, None);
        assert!(result.is_err());
    }

    #[test]
    fn test_upsert_invalid_column_name() {
        let qb = QueryBuilder::new("users").unwrap();
        let values = vec![
            ("drop".to_string(), ExtractedValue::String("value".to_string())),
        ];
        let conflict_target = vec!["drop".to_string()];
        let result = qb.build_upsert(&values, &conflict_target, None);
        assert!(result.is_err());
    }

    #[test]
    fn test_inner_join() {
        let condition = JoinCondition::new("author_id", "users", "id").unwrap();
        let qb = QueryBuilder::new("posts").unwrap()
            .select(vec!["posts.id".to_string(), "posts.title".to_string(), "users.name".to_string()]).unwrap()
            .inner_join("users", None, condition).unwrap();

        let (sql, _) = qb.build_select();
        assert!(sql.contains("INNER JOIN \"users\" ON \"posts\".\"author_id\" = \"users\".\"id\""));
        assert_eq!(sql, "SELECT \"posts\".\"id\", \"posts\".\"title\", \"users\".\"name\" FROM \"posts\" INNER JOIN \"users\" ON \"posts\".\"author_id\" = \"users\".\"id\"");
    }

    #[test]
    fn test_left_join_with_alias() {
        let condition = JoinCondition::new("author_id", "u", "id").unwrap();
        let qb = QueryBuilder::new("posts").unwrap()
            .select(vec!["p.id".to_string(), "u.name".to_string()]).unwrap()
            .left_join("users", Some("u"), condition).unwrap();

        let (sql, _) = qb.build_select();
        assert!(sql.contains("LEFT JOIN \"users\" AS \"u\" ON \"posts\".\"author_id\" = \"u\".\"id\""));
        assert_eq!(sql, "SELECT \"p\".\"id\", \"u\".\"name\" FROM \"posts\" LEFT JOIN \"users\" AS \"u\" ON \"posts\".\"author_id\" = \"u\".\"id\"");
    }

    #[test]
    fn test_multiple_joins() {
        let condition1 = JoinCondition::new("author_id", "u", "id").unwrap();
        let condition2 = JoinCondition::new("category_id", "c", "id").unwrap();
        let qb = QueryBuilder::new("posts").unwrap()
            .inner_join("users", Some("u"), condition1).unwrap()
            .left_join("categories", Some("c"), condition2).unwrap();

        let (sql, _) = qb.build_select();
        assert!(sql.contains("INNER JOIN"));
        assert!(sql.contains("LEFT JOIN"));
        assert_eq!(sql, "SELECT * FROM \"posts\" INNER JOIN \"users\" AS \"u\" ON \"posts\".\"author_id\" = \"u\".\"id\" LEFT JOIN \"categories\" AS \"c\" ON \"posts\".\"category_id\" = \"c\".\"id\"");
    }

    #[test]
    fn test_right_join() {
        let condition = JoinCondition::new("author_id", "users", "id").unwrap();
        let qb = QueryBuilder::new("posts").unwrap()
            .select(vec!["posts.id".to_string(), "users.name".to_string()]).unwrap()
            .right_join("users", None, condition).unwrap();

        let (sql, _) = qb.build_select();
        assert!(sql.contains("RIGHT JOIN \"users\" ON \"posts\".\"author_id\" = \"users\".\"id\""));
    }

    #[test]
    fn test_full_join() {
        let condition = JoinCondition::new("author_id", "users", "id").unwrap();
        let qb = QueryBuilder::new("posts").unwrap()
            .select(vec!["posts.id".to_string(), "users.name".to_string()]).unwrap()
            .full_join("users", None, condition).unwrap();

        let (sql, _) = qb.build_select();
        assert!(sql.contains("FULL OUTER JOIN \"users\" ON \"posts\".\"author_id\" = \"users\".\"id\""));
    }

    #[test]
    fn test_join_with_where() {
        let condition = JoinCondition::new("author_id", "u", "id").unwrap();
        let qb = QueryBuilder::new("posts").unwrap()
            .select(vec!["posts.id".to_string(), "posts.title".to_string(), "users.name".to_string()]).unwrap()
            .inner_join("users", Some("u"), condition).unwrap()
            .where_clause("posts.published", Operator::Eq, ExtractedValue::Bool(true)).unwrap();

        let (sql, params) = qb.build_select();
        assert!(sql.contains("INNER JOIN"));
        assert!(sql.contains("WHERE \"posts\".\"published\" = $1"));
        assert_eq!(params.len(), 1);
    }

    #[test]
    fn test_join_with_order_and_limit() {
        let condition = JoinCondition::new("author_id", "u", "id").unwrap();
        let qb = QueryBuilder::new("posts").unwrap()
            .select(vec!["posts.id".to_string(), "users.name".to_string()]).unwrap()
            .inner_join("users", Some("u"), condition).unwrap()
            .order_by("posts.created_at", OrderDirection::Desc).unwrap()
            .limit(10);

        let (sql, params) = qb.build_select();
        assert!(sql.contains("INNER JOIN"));
        assert!(sql.contains("ORDER BY \"posts\".\"created_at\" DESC"));
        assert!(sql.contains("LIMIT $1"));
        assert_eq!(params.len(), 1);
    }

    #[test]
    fn test_join_invalid_table_name() {
        let result = QueryBuilder::new("posts").unwrap()
            .inner_join("drop", None, JoinCondition::new("author_id", "users", "id").unwrap());
        assert!(result.is_err());
    }

    #[test]
    fn test_join_invalid_alias() {
        let result = QueryBuilder::new("posts").unwrap()
            .inner_join("users", Some("select"), JoinCondition::new("author_id", "users", "id").unwrap());
        assert!(result.is_err());
    }

    #[test]
    fn test_join_with_schema_qualified_table() {
        let condition = JoinCondition::new("author_id", "u", "id").unwrap();
        let qb = QueryBuilder::new("public.posts").unwrap()
            .select(vec!["p.id".to_string(), "u.name".to_string()]).unwrap()
            .inner_join("public.users", Some("u"), condition).unwrap();

        let (sql, _) = qb.build_select();
        assert!(sql.contains("FROM \"public\".\"posts\""));
        assert!(sql.contains("INNER JOIN \"public\".\"users\" AS \"u\""));
    }

    #[test]
    fn test_join_condition_valid() {
        let condition = JoinCondition::new("author_id", "users", "id").unwrap();
        let sql = condition.to_sql("posts");
        assert_eq!(sql, "\"posts\".\"author_id\" = \"users\".\"id\"");
    }

    #[test]
    fn test_join_condition_with_alias() {
        let condition = JoinCondition::new("author_id", "u", "id").unwrap();
        let sql = condition.to_sql("posts");
        assert_eq!(sql, "\"posts\".\"author_id\" = \"u\".\"id\"");
    }

    #[test]
    fn test_join_condition_invalid_left_column() {
        let result = JoinCondition::new("drop", "users", "id");
        assert!(result.is_err());
    }

    #[test]
    fn test_join_condition_invalid_right_table() {
        let result = JoinCondition::new("author_id", "select", "id");
        assert!(result.is_err());
    }

    #[test]
    fn test_join_condition_invalid_right_column() {
        let result = JoinCondition::new("author_id", "users", "delete");
        assert!(result.is_err());
    }

    #[test]
    fn test_join_condition_sql_injection_attempt() {
        // Attempt SQL injection in column name
        let result = JoinCondition::new("author_id; DROP TABLE users--", "users", "id");
        assert!(result.is_err());

        // Attempt SQL injection in table name
        let result = JoinCondition::new("author_id", "users; DROP TABLE posts--", "id");
        assert!(result.is_err());

        // Attempt SQL injection in right column
        let result = JoinCondition::new("author_id", "users", "id OR 1=1--");
        assert!(result.is_err());
    }

    #[test]
    fn test_aggregate_count_all() {
        let qb = QueryBuilder::new("users").unwrap()
            .aggregate(AggregateFunction::Count, None).unwrap();
        let (sql, params) = qb.build_select();
        assert_eq!(sql, "SELECT COUNT(*) FROM \"users\"");
        assert_eq!(params.len(), 0);
    }

    #[test]
    fn test_aggregate_count_with_alias() {
        let qb = QueryBuilder::new("users").unwrap()
            .aggregate(AggregateFunction::Count, Some("total")).unwrap();
        let (sql, params) = qb.build_select();
        assert_eq!(sql, "SELECT COUNT(*) AS \"total\" FROM \"users\"");
        assert_eq!(params.len(), 0);
    }

    #[test]
    fn test_aggregate_count_column() {
        let qb = QueryBuilder::new("users").unwrap()
            .aggregate(AggregateFunction::CountColumn("email".to_string()), Some("email_count")).unwrap();
        let (sql, params) = qb.build_select();
        assert_eq!(sql, "SELECT COUNT(\"email\") AS \"email_count\" FROM \"users\"");
        assert_eq!(params.len(), 0);
    }

    #[test]
    fn test_aggregate_count_distinct() {
        let qb = QueryBuilder::new("orders").unwrap()
            .aggregate(AggregateFunction::CountDistinct("customer_id".to_string()), Some("unique_customers")).unwrap();
        let (sql, params) = qb.build_select();
        assert_eq!(sql, "SELECT COUNT(DISTINCT \"customer_id\") AS \"unique_customers\" FROM \"orders\"");
        assert_eq!(params.len(), 0);
    }

    #[test]
    fn test_aggregate_sum() {
        let qb = QueryBuilder::new("orders").unwrap()
            .aggregate(AggregateFunction::Sum("total".to_string()), Some("revenue")).unwrap();
        let (sql, params) = qb.build_select();
        assert_eq!(sql, "SELECT SUM(\"total\") AS \"revenue\" FROM \"orders\"");
        assert_eq!(params.len(), 0);
    }

    #[test]
    fn test_aggregate_avg() {
        let qb = QueryBuilder::new("products").unwrap()
            .aggregate(AggregateFunction::Avg("price".to_string()), Some("avg_price")).unwrap();
        let (sql, params) = qb.build_select();
        assert_eq!(sql, "SELECT AVG(\"price\") AS \"avg_price\" FROM \"products\"");
        assert_eq!(params.len(), 0);
    }

    #[test]
    fn test_aggregate_min_max() {
        let qb = QueryBuilder::new("temperatures").unwrap()
            .aggregate(AggregateFunction::Min("value".to_string()), Some("min_temp")).unwrap()
            .aggregate(AggregateFunction::Max("value".to_string()), Some("max_temp")).unwrap();
        let (sql, params) = qb.build_select();
        assert_eq!(sql, "SELECT MIN(\"value\") AS \"min_temp\", MAX(\"value\") AS \"max_temp\" FROM \"temperatures\"");
        assert_eq!(params.len(), 0);
    }

    #[test]
    fn test_aggregate_with_group_by() {
        let qb = QueryBuilder::new("sales").unwrap()
            .aggregate(AggregateFunction::Sum("amount".to_string()), Some("total_sales")).unwrap()
            .group_by(&["region", "product"]).unwrap();
        let (sql, params) = qb.build_select();
        assert_eq!(sql, "SELECT \"region\", \"product\", SUM(\"amount\") AS \"total_sales\" FROM \"sales\" GROUP BY \"region\", \"product\"");
        assert_eq!(params.len(), 0);
    }

    #[test]
    fn test_aggregate_with_where() {
        let qb = QueryBuilder::new("orders").unwrap()
            .aggregate(AggregateFunction::Count, Some("order_count")).unwrap()
            .where_clause("status", Operator::Eq, ExtractedValue::String("completed".to_string())).unwrap();
        let (sql, params) = qb.build_select();
        assert_eq!(sql, "SELECT COUNT(*) AS \"order_count\" FROM \"orders\" WHERE \"status\" = $1");
        assert_eq!(params.len(), 1);
    }

    #[test]
    fn test_aggregate_with_where_and_group_by() {
        let qb = QueryBuilder::new("sales").unwrap()
            .aggregate(AggregateFunction::Sum("revenue".to_string()), Some("total")).unwrap()
            .aggregate(AggregateFunction::Avg("revenue".to_string()), Some("average")).unwrap()
            .where_clause("year", Operator::Eq, ExtractedValue::Int(2024)).unwrap()
            .group_by(&["department"]).unwrap();
        let (sql, params) = qb.build_select();
        assert_eq!(
            sql,
            "SELECT \"department\", SUM(\"revenue\") AS \"total\", AVG(\"revenue\") AS \"average\" FROM \"sales\" WHERE \"year\" = $1 GROUP BY \"department\""
        );
        assert_eq!(params.len(), 1);
    }

    #[test]
    fn test_aggregate_with_group_by_and_order_by() {
        let qb = QueryBuilder::new("sales").unwrap()
            .aggregate(AggregateFunction::Sum("amount".to_string()), Some("total")).unwrap()
            .group_by(&["category"]).unwrap()
            .order_by("total", OrderDirection::Desc).unwrap();
        let (sql, params) = qb.build_select();
        assert_eq!(
            sql,
            "SELECT \"category\", SUM(\"amount\") AS \"total\" FROM \"sales\" GROUP BY \"category\" ORDER BY \"total\" DESC"
        );
        assert_eq!(params.len(), 0);
    }

    #[test]
    fn test_aggregate_with_limit() {
        let qb = QueryBuilder::new("products").unwrap()
            .aggregate(AggregateFunction::Count, None).unwrap()
            .limit(1);
        let (sql, params) = qb.build_select();
        assert_eq!(sql, "SELECT COUNT(*) FROM \"products\" LIMIT $1");
        assert_eq!(params.len(), 1);
    }

    #[test]
    fn test_clear_aggregates() {
        let qb = QueryBuilder::new("users").unwrap()
            .aggregate(AggregateFunction::Count, Some("total")).unwrap()
            .group_by(&["department"]).unwrap()
            .clear_aggregates();
        let (sql, params) = qb.build_select();
        assert_eq!(sql, "SELECT * FROM \"users\"");
        assert_eq!(params.len(), 0);
    }

    #[test]
    fn test_aggregate_invalid_column_name() {
        let result = QueryBuilder::new("users").unwrap()
            .aggregate(AggregateFunction::Sum("drop".to_string()), None);
        assert!(result.is_err());
    }

    #[test]
    fn test_aggregate_invalid_alias() {
        let result = QueryBuilder::new("users").unwrap()
            .aggregate(AggregateFunction::Count, Some("select"));
        assert!(result.is_err());
    }

    #[test]
    fn test_group_by_invalid_column() {
        let result = QueryBuilder::new("users").unwrap()
            .group_by(&["department", "select"]);
        assert!(result.is_err());
    }

    #[test]
    fn test_multiple_aggregates_no_alias() {
        let qb = QueryBuilder::new("sales").unwrap()
            .aggregate(AggregateFunction::Count, None).unwrap()
            .aggregate(AggregateFunction::Sum("amount".to_string()), None).unwrap()
            .aggregate(AggregateFunction::Avg("amount".to_string()), None).unwrap();
        let (sql, params) = qb.build_select();
        assert_eq!(sql, "SELECT COUNT(*), SUM(\"amount\"), AVG(\"amount\") FROM \"sales\"");
        assert_eq!(params.len(), 0);
    }

    #[test]
    fn test_complex_aggregate_query() {
        let qb = QueryBuilder::new("orders").unwrap()
            .aggregate(AggregateFunction::Count, Some("total_orders")).unwrap()
            .aggregate(AggregateFunction::Sum("amount".to_string()), Some("total_revenue")).unwrap()
            .aggregate(AggregateFunction::Avg("amount".to_string()), Some("avg_order_value")).unwrap()
            .aggregate(AggregateFunction::CountDistinct("customer_id".to_string()), Some("unique_customers")).unwrap()
            .where_clause("status", Operator::In, ExtractedValue::Array(vec![
                ExtractedValue::String("completed".to_string()),
                ExtractedValue::String("shipped".to_string()),
            ])).unwrap()
            .where_clause("created_at", Operator::Gte, ExtractedValue::String("2024-01-01".to_string())).unwrap()
            .group_by(&["region", "product_category"]).unwrap()
            .order_by("total_revenue", OrderDirection::Desc).unwrap()
            .limit(10);
        let (sql, params) = qb.build_select();
        assert_eq!(
            sql,
            "SELECT \"region\", \"product_category\", COUNT(*) AS \"total_orders\", SUM(\"amount\") AS \"total_revenue\", AVG(\"amount\") AS \"avg_order_value\", COUNT(DISTINCT \"customer_id\") AS \"unique_customers\" FROM \"orders\" WHERE \"status\" IN ($1) AND \"created_at\" >= $2 GROUP BY \"region\", \"product_category\" ORDER BY \"total_revenue\" DESC LIMIT $3"
        );
        assert_eq!(params.len(), 3);
    }

    #[test]
    fn test_having_basic() {
        let qb = QueryBuilder::new("orders").unwrap()
            .aggregate(AggregateFunction::Count, Some("order_count")).unwrap()
            .group_by(&["status"]).unwrap()
            .having(AggregateFunction::Count, Operator::Gt, ExtractedValue::Int(10)).unwrap();

        let (sql, params) = qb.build_select();
        assert!(sql.contains("HAVING COUNT(*) > $"));
        assert_eq!(params.len(), 1);
        match &params[0] {
            ExtractedValue::Int(10) => {},
            _ => panic!("Expected Int(10)"),
        }
    }

    #[test]
    fn test_having_with_sum() {
        let qb = QueryBuilder::new("orders").unwrap()
            .aggregate(AggregateFunction::Sum("amount".to_string()), Some("total")).unwrap()
            .group_by(&["customer_id"]).unwrap()
            .having(AggregateFunction::Sum("amount".to_string()), Operator::Gte, ExtractedValue::Float(1000.0)).unwrap();

        let (sql, params) = qb.build_select();
        assert!(sql.contains("HAVING SUM(\"amount\") >= $"));
        assert_eq!(params.len(), 1);
    }

    #[test]
    fn test_having_multiple_conditions() {
        let qb = QueryBuilder::new("orders").unwrap()
            .aggregate(AggregateFunction::Count, None).unwrap()
            .aggregate(AggregateFunction::Sum("amount".to_string()), None).unwrap()
            .group_by(&["status"]).unwrap()
            .having(AggregateFunction::Count, Operator::Gt, ExtractedValue::Int(5)).unwrap()
            .having(AggregateFunction::Sum("amount".to_string()), Operator::Lt, ExtractedValue::Float(10000.0)).unwrap();

        let (sql, params) = qb.build_select();
        assert!(sql.contains("HAVING COUNT(*) > $"));
        assert!(sql.contains(" AND SUM(\"amount\") < $"));
        assert_eq!(params.len(), 2);
    }

    #[test]
    fn test_having_with_where() {
        let qb = QueryBuilder::new("orders").unwrap()
            .aggregate(AggregateFunction::Count, Some("order_count")).unwrap()
            .where_clause("year", Operator::Eq, ExtractedValue::Int(2024)).unwrap()
            .group_by(&["status"]).unwrap()
            .having(AggregateFunction::Count, Operator::Gte, ExtractedValue::Int(100)).unwrap();

        let (sql, params) = qb.build_select();
        assert!(sql.contains("WHERE \"year\" = $1"));
        assert!(sql.contains("HAVING COUNT(*) >= $2"));
        assert_eq!(params.len(), 2);
    }

    #[test]
    fn test_having_with_avg() {
        let qb = QueryBuilder::new("students").unwrap()
            .aggregate(AggregateFunction::Avg("score".to_string()), Some("avg_score")).unwrap()
            .group_by(&["class_id"]).unwrap()
            .having(AggregateFunction::Avg("score".to_string()), Operator::Gt, ExtractedValue::Float(75.0)).unwrap();

        let (sql, params) = qb.build_select();
        assert!(sql.contains("HAVING AVG(\"score\") > $"));
        assert_eq!(params.len(), 1);
    }

    #[test]
    fn test_having_with_count_distinct() {
        let qb = QueryBuilder::new("orders").unwrap()
            .aggregate(AggregateFunction::CountDistinct("customer_id".to_string()), Some("unique_customers")).unwrap()
            .group_by(&["region"]).unwrap()
            .having(AggregateFunction::CountDistinct("customer_id".to_string()), Operator::Gte, ExtractedValue::Int(50)).unwrap();

        let (sql, params) = qb.build_select();
        assert!(sql.contains("HAVING COUNT(DISTINCT \"customer_id\") >= $"));
        assert_eq!(params.len(), 1);
    }

    #[test]
    fn test_having_with_min_max() {
        let qb = QueryBuilder::new("products").unwrap()
            .aggregate(AggregateFunction::Max("price".to_string()), Some("max_price")).unwrap()
            .group_by(&["category"]).unwrap()
            .having(AggregateFunction::Max("price".to_string()), Operator::Lt, ExtractedValue::Float(1000.0)).unwrap();

        let (sql, params) = qb.build_select();
        assert!(sql.contains("HAVING MAX(\"price\") < $"));
        assert_eq!(params.len(), 1);
    }

    #[test]
    fn test_clear_having() {
        let qb = QueryBuilder::new("orders").unwrap()
            .aggregate(AggregateFunction::Count, Some("order_count")).unwrap()
            .group_by(&["status"]).unwrap()
            .having(AggregateFunction::Count, Operator::Gt, ExtractedValue::Int(10)).unwrap()
            .clear_having();

        let (sql, params) = qb.build_select();
        assert!(!sql.contains("HAVING"));
        assert_eq!(params.len(), 0);
    }

    #[test]
    fn test_having_invalid_column() {
        let result = QueryBuilder::new("orders").unwrap()
            .aggregate(AggregateFunction::Count, None).unwrap()
            .group_by(&["status"]).unwrap()
            .having(AggregateFunction::Sum("drop".to_string()), Operator::Gt, ExtractedValue::Int(100));
        assert!(result.is_err());
    }

    #[test]
    fn test_complex_having_query() {
        let qb = QueryBuilder::new("sales").unwrap()
            .aggregate(AggregateFunction::Sum("revenue".to_string()), Some("total_revenue")).unwrap()
            .aggregate(AggregateFunction::Count, Some("sale_count")).unwrap()
            .aggregate(AggregateFunction::Avg("revenue".to_string()), Some("avg_revenue")).unwrap()
            .where_clause("year", Operator::Eq, ExtractedValue::Int(2024)).unwrap()
            .group_by(&["region", "category"]).unwrap()
            .having(AggregateFunction::Sum("revenue".to_string()), Operator::Gte, ExtractedValue::Float(100000.0)).unwrap()
            .having(AggregateFunction::Count, Operator::Gt, ExtractedValue::Int(10)).unwrap()
            .order_by("total_revenue", OrderDirection::Desc).unwrap()
            .limit(20);

        let (sql, params) = qb.build_select();
        assert!(sql.contains("SELECT \"region\", \"category\", SUM(\"revenue\") AS \"total_revenue\", COUNT(*) AS \"sale_count\", AVG(\"revenue\") AS \"avg_revenue\""));
        assert!(sql.contains("WHERE \"year\" = $1"));
        assert!(sql.contains("GROUP BY \"region\", \"category\""));
        assert!(sql.contains("HAVING SUM(\"revenue\") >= $2 AND COUNT(*) > $3"));
        assert!(sql.contains("ORDER BY \"total_revenue\" DESC"));
        assert!(sql.contains("LIMIT $4"));
        assert_eq!(params.len(), 4);
    }

    #[test]
    fn test_distinct_basic() {
        let qb = QueryBuilder::new("users").unwrap()
            .select(vec!["email".to_string()]).unwrap()
            .distinct();
        let (sql, _) = qb.build_select();
        assert!(sql.starts_with("SELECT DISTINCT "));
        assert!(sql.contains("\"email\""));
    }

    #[test]
    fn test_distinct_on_single_column() {
        let qb = QueryBuilder::new("orders").unwrap()
            .distinct_on(&["user_id"]).unwrap()
            .order_by("user_id", OrderDirection::Asc).unwrap();
        let (sql, _) = qb.build_select();
        assert!(sql.contains("DISTINCT ON (\"user_id\")"));
    }

    #[test]
    fn test_distinct_on_multiple_columns() {
        let qb = QueryBuilder::new("orders").unwrap()
            .distinct_on(&["user_id", "status"]).unwrap();
        let (sql, _) = qb.build_select();
        assert!(sql.contains("DISTINCT ON (\"user_id\", \"status\")"));
    }

    #[test]
    fn test_distinct_on_validates_columns() {
        let result = QueryBuilder::new("orders").unwrap()
            .distinct_on(&["user_id; DROP TABLE orders"]);
        assert!(result.is_err());
    }

    #[test]
    fn test_distinct_with_where_and_order() {
        let qb = QueryBuilder::new("users").unwrap()
            .select(vec!["email".to_string(), "name".to_string()]).unwrap()
            .distinct()
            .where_clause("active", Operator::Eq, ExtractedValue::Bool(true)).unwrap()
            .order_by("email", OrderDirection::Asc).unwrap();
        let (sql, _) = qb.build_select();
        assert!(sql.starts_with("SELECT DISTINCT "));
        assert!(sql.contains("WHERE"));
        assert!(sql.contains("ORDER BY"));
    }

    #[test]
    fn test_distinct_on_takes_precedence_over_distinct() {
        let qb = QueryBuilder::new("orders").unwrap()
            .distinct()
            .distinct_on(&["user_id"]).unwrap();
        let (sql, _) = qb.build_select();
        assert!(sql.contains("DISTINCT ON (\"user_id\")"));
        assert!(!sql.contains("SELECT DISTINCT DISTINCT ON"));
    }

    #[test]
    fn test_clear_distinct() {
        let qb = QueryBuilder::new("users").unwrap()
            .select(vec!["email".to_string()]).unwrap()
            .distinct()
            .clear_distinct();
        let (sql, _) = qb.build_select();
        assert!(!sql.contains("DISTINCT"));
    }

    #[test]
    fn test_clear_distinct_on() {
        let qb = QueryBuilder::new("orders").unwrap()
            .distinct_on(&["user_id"]).unwrap()
            .clear_distinct();
        let (sql, _) = qb.build_select();
        assert!(!sql.contains("DISTINCT ON"));
    }

    #[test]
    fn test_cte_basic() {
        let cte_query = QueryBuilder::new("orders").unwrap()
            .aggregate(AggregateFunction::Sum("amount".to_string()), Some("total")).unwrap()
            .group_by(&["user_id"]).unwrap();

        let main_query = QueryBuilder::new("order_totals").unwrap()
            .with_cte("order_totals", cte_query).unwrap();

        let (sql, _) = main_query.build_select();
        assert!(sql.starts_with("WITH "));
        assert!(sql.contains("\"order_totals\" AS ("));
        assert!(sql.contains("SELECT \"user_id\", SUM(\"amount\") AS \"total\""));
        assert!(sql.contains("FROM \"orders\""));
        assert!(sql.contains("GROUP BY \"user_id\""));
        assert!(sql.contains(") SELECT * FROM \"order_totals\""));
    }

    #[test]
    fn test_cte_with_where() {
        let cte_query = QueryBuilder::new("orders").unwrap()
            .select(vec!["user_id".to_string(), "amount".to_string()]).unwrap()
            .where_clause("status", Operator::Eq, ExtractedValue::String("completed".to_string())).unwrap();

        let main_query = QueryBuilder::new("completed_orders").unwrap()
            .with_cte("completed_orders", cte_query).unwrap()
            .where_clause("amount", Operator::Gt, ExtractedValue::Float(100.0)).unwrap();

        let (sql, params) = main_query.build_select();
        assert!(sql.contains("WITH"));
        assert!(sql.contains("\"completed_orders\" AS ("));
        assert_eq!(params.len(), 2); // One from CTE, one from main query

        // First param is from CTE (status = 'completed')
        match &params[0] {
            ExtractedValue::String(s) => assert_eq!(s, "completed"),
            _ => panic!("Expected String for first param"),
        }

        // Second param is from main query (amount > 100.0)
        match &params[1] {
            ExtractedValue::Float(f) => assert_eq!(*f, 100.0),
            _ => panic!("Expected Float for second param"),
        }
    }

    #[test]
    fn test_cte_name_validation() {
        let cte_query = QueryBuilder::new("orders").unwrap();
        let result = QueryBuilder::new("test").unwrap()
            .with_cte("invalid; DROP TABLE", cte_query);
        assert!(result.is_err());
        if let Err(e) = result {
            // Should contain error about invalid characters (semicolon and space)
            let error_msg = format!("{:?}", e);
            assert!(error_msg.contains("invalid character") || error_msg.contains("contains invalid"));
        }
    }

    #[test]
    fn test_multiple_ctes() {
        let cte1 = QueryBuilder::new("orders").unwrap()
            .where_clause("status", Operator::Eq, ExtractedValue::String("completed".to_string())).unwrap();
        let cte2 = QueryBuilder::new("users").unwrap()
            .where_clause("active", Operator::Eq, ExtractedValue::Bool(true)).unwrap();

        let main = QueryBuilder::new("completed_orders").unwrap()
            .with_cte("completed_orders", cte1).unwrap()
            .with_cte("active_users", cte2).unwrap();

        let (sql, params) = main.build_select();
        assert!(sql.contains("WITH \"completed_orders\" AS"));
        assert!(sql.contains("\"active_users\" AS"));
        assert_eq!(params.len(), 2);

        // Verify params are in correct order
        match &params[0] {
            ExtractedValue::String(s) => assert_eq!(s, "completed"),
            _ => panic!("Expected String for first param"),
        }
        match &params[1] {
            ExtractedValue::Bool(b) => assert_eq!(*b, true),
            _ => panic!("Expected Bool for second param"),
        }
    }

    #[test]
    fn test_cte_with_raw_sql() {
        let main_query = QueryBuilder::new("high_value").unwrap()
            .with_cte_raw(
                "high_value",
                "SELECT user_id, SUM(amount) as total FROM orders GROUP BY user_id HAVING SUM(amount) > $1",
                vec![ExtractedValue::Float(1000.0)]
            ).unwrap()
            .where_clause("total", Operator::Gt, ExtractedValue::Float(5000.0)).unwrap();

        let (sql, params) = main_query.build_select();
        assert!(sql.starts_with("WITH "));
        assert!(sql.contains("\"high_value\" AS ("));
        assert_eq!(params.len(), 2);
    }

    #[test]
    fn test_clear_ctes() {
        let cte_query = QueryBuilder::new("orders").unwrap();
        let main_query = QueryBuilder::new("test").unwrap()
            .with_cte("cte1", cte_query).unwrap()
            .clear_ctes();

        let (sql, _) = main_query.build_select();
        assert!(!sql.contains("WITH"));
    }

    #[test]
    fn test_cte_parameter_adjustment() {
        // Test that parameter indices are correctly adjusted when combining CTEs
        let cte1 = QueryBuilder::new("t1").unwrap()
            .where_clause("a", Operator::Eq, ExtractedValue::Int(1)).unwrap();
        let cte2 = QueryBuilder::new("t2").unwrap()
            .where_clause("b", Operator::Eq, ExtractedValue::Int(2)).unwrap();

        let main = QueryBuilder::new("result").unwrap()
            .with_cte("cte1", cte1).unwrap()
            .with_cte("cte2", cte2).unwrap()
            .where_clause("c", Operator::Eq, ExtractedValue::Int(3)).unwrap();

        let (sql, params) = main.build_select();

        // The SQL should have properly adjusted parameter indices
        // CTE1 uses $1, CTE2 should use $2, main query should use $3
        assert!(sql.contains("WITH \"cte1\" AS (SELECT * FROM \"t1\" WHERE \"a\" = $1), \"cte2\" AS (SELECT * FROM \"t2\" WHERE \"b\" = $2)"));
        assert!(sql.contains("WHERE \"c\" = $3"));

        assert_eq!(params.len(), 3);
        assert_eq!(params[0], ExtractedValue::Int(1));
        assert_eq!(params[1], ExtractedValue::Int(2));
        assert_eq!(params[2], ExtractedValue::Int(3));
    }

    #[test]
    fn test_where_in_subquery() {
        let subquery = QueryBuilder::new("orders").unwrap()
            .select(vec!["user_id".to_string()]).unwrap()
            .where_clause("total", Operator::Gt, ExtractedValue::Float(1000.0)).unwrap();

        let main = QueryBuilder::new("users").unwrap()
            .where_in_subquery("id", subquery).unwrap();

        let (sql, params) = main.build_select();
        assert!(sql.contains("WHERE \"id\" IN (SELECT"));
        assert!(sql.contains("\"user_id\""));
        assert!(sql.contains("\"total\" > $1"));
        assert_eq!(params.len(), 1);
        assert_eq!(params[0], ExtractedValue::Float(1000.0));
    }

    #[test]
    fn test_where_not_in_subquery() {
        let subquery = QueryBuilder::new("inactive_users").unwrap()
            .select(vec!["user_id".to_string()]).unwrap();

        let main = QueryBuilder::new("users").unwrap()
            .where_not_in_subquery("id", subquery).unwrap();

        let (sql, _) = main.build_select();
        assert!(sql.contains("WHERE \"id\" NOT IN (SELECT"));
        assert!(sql.contains("\"user_id\""));
    }

    #[test]
    fn test_where_exists() {
        let subquery = QueryBuilder::new("orders").unwrap()
            .where_clause("user_id", Operator::Eq, ExtractedValue::Int(1)).unwrap();

        let main = QueryBuilder::new("users").unwrap()
            .where_exists(subquery).unwrap();

        let (sql, params) = main.build_select();
        assert!(sql.contains("WHERE EXISTS (SELECT"));
        assert!(sql.contains("\"user_id\" = $1"));
        assert_eq!(params.len(), 1);
    }

    #[test]
    fn test_where_not_exists() {
        let subquery = QueryBuilder::new("orders").unwrap()
            .where_clause("user_id", Operator::Eq, ExtractedValue::Int(1)).unwrap();

        let main = QueryBuilder::new("users").unwrap()
            .where_not_exists(subquery).unwrap();

        let (sql, params) = main.build_select();
        assert!(sql.contains("WHERE NOT EXISTS (SELECT"));
        assert_eq!(params.len(), 1);
    }

    #[test]
    fn test_subquery_with_multiple_params() {
        let subquery = QueryBuilder::new("orders").unwrap()
            .where_clause("total", Operator::Gt, ExtractedValue::Float(100.0)).unwrap()
            .where_clause("status", Operator::Eq, ExtractedValue::String("completed".to_string())).unwrap();

        let main = QueryBuilder::new("users").unwrap()
            .where_clause("active", Operator::Eq, ExtractedValue::Bool(true)).unwrap()
            .where_in_subquery("id", subquery).unwrap();

        let (sql, params) = main.build_select();
        assert_eq!(params.len(), 3); // 1 from main + 2 from subquery
        // Main query param comes first
        assert_eq!(params[0], ExtractedValue::Bool(true));
        // Then subquery params
        assert_eq!(params[1], ExtractedValue::Float(100.0));
        assert_eq!(params[2], ExtractedValue::String("completed".to_string()));

        // Verify parameter indices are adjusted correctly
        assert!(sql.contains("\"active\" = $1"));
        assert!(sql.contains("\"total\" > $2"));
        assert!(sql.contains("\"status\" = $3"));
    }

    #[test]
    fn test_subquery_parameter_index_adjustment() {
        // Test that subquery parameter indices are correctly adjusted
        let subquery = QueryBuilder::new("orders").unwrap()
            .where_clause("total", Operator::Gt, ExtractedValue::Float(500.0)).unwrap();

        let main = QueryBuilder::new("users").unwrap()
            .where_clause("age", Operator::Gte, ExtractedValue::Int(18)).unwrap()
            .where_clause("status", Operator::Eq, ExtractedValue::String("active".to_string())).unwrap()
            .where_in_subquery("id", subquery).unwrap();

        let (sql, params) = main.build_select();

        // Should have 3 params: age, status, total
        assert_eq!(params.len(), 3);

        // Main query params should use $1 and $2
        assert!(sql.contains("\"age\" >= $1"));
        assert!(sql.contains("\"status\" = $2"));

        // Subquery param should be adjusted to $3
        assert!(sql.contains("\"total\" > $3"));
    }

    #[test]
    fn test_multiple_subqueries() {
        let subquery1 = QueryBuilder::new("orders").unwrap()
            .select(vec!["user_id".to_string()]).unwrap()
            .where_clause("total", Operator::Gt, ExtractedValue::Float(1000.0)).unwrap();

        let subquery2 = QueryBuilder::new("banned_users").unwrap()
            .select(vec!["user_id".to_string()]).unwrap();

        let main = QueryBuilder::new("users").unwrap()
            .where_in_subquery("id", subquery1).unwrap()
            .where_not_in_subquery("id", subquery2).unwrap();

        let (sql, params) = main.build_select();
        assert!(sql.contains("\"id\" IN (SELECT"));
        assert!(sql.contains("\"id\" NOT IN (SELECT"));
        assert_eq!(params.len(), 1);
    }

#[test]
fn debug_cte_sql() {
    let cte_query = QueryBuilder::new("orders").unwrap()
        .aggregate(AggregateFunction::Sum("amount".to_string()), Some("total")).unwrap()
        .group_by(&["user_id"]).unwrap();

    let main_query = QueryBuilder::new("order_totals").unwrap()
        .with_cte("order_totals", cte_query).unwrap()
        .where_clause("total", Operator::Gt, ExtractedValue::Float(1000.0)).unwrap();

    let (sql, params) = main_query.build_select();
    eprintln!("Generated SQL:\n{}", sql);
    eprintln!("Parameters: {:?}", params);

    // Verify the structure
    assert!(sql.starts_with("WITH "));
    assert!(sql.contains("\"order_totals\" AS ("));
}

#[test]
fn test_window_row_number() {
    let qb = QueryBuilder::new("orders")
        .unwrap()
        .window(
            WindowFunction::RowNumber,
            WindowSpec::new().order_by("amount", OrderDirection::Desc),
            "rank",
        )
        .unwrap();
    let (sql, _) = qb.build_select();
    assert!(sql.contains("ROW_NUMBER() OVER (ORDER BY \"amount\" DESC) AS \"rank\""));
}

#[test]
fn test_window_with_partition() {
    let qb = QueryBuilder::new("orders")
        .unwrap()
        .window(
            WindowFunction::Sum("amount".to_string()),
            WindowSpec::new()
                .partition_by(&["user_id"])
                .order_by("created_at", OrderDirection::Asc),
            "running_total",
        )
        .unwrap();
    let (sql, _) = qb.build_select();
    assert!(sql.contains("SUM(\"amount\") OVER"));
    assert!(sql.contains("PARTITION BY \"user_id\""));
    assert!(sql.contains("ORDER BY \"created_at\" ASC"));
    assert!(sql.contains("AS \"running_total\""));
}

#[test]
fn test_window_lag() {
    let qb = QueryBuilder::new("orders")
        .unwrap()
        .window(
            WindowFunction::Lag("amount".to_string(), Some(1), None),
            WindowSpec::new().order_by("created_at", OrderDirection::Asc),
            "prev_amount",
        )
        .unwrap();
    let (sql, _) = qb.build_select();
    assert!(sql.contains("LAG(\"amount\", 1) OVER"));
    assert!(sql.contains("ORDER BY \"created_at\" ASC"));
    assert!(sql.contains("AS \"prev_amount\""));
}

#[test]
fn test_window_multiple_functions() {
    let qb = QueryBuilder::new("orders")
        .unwrap()
        .window(
            WindowFunction::RowNumber,
            WindowSpec::new().order_by("amount", OrderDirection::Desc),
            "rank",
        )
        .unwrap()
        .window(
            WindowFunction::Sum("amount".to_string()),
            WindowSpec::new().partition_by(&["user_id"]),
            "user_total",
        )
        .unwrap();
    let (sql, _) = qb.build_select();
    assert!(sql.contains("ROW_NUMBER()"));
    assert!(sql.contains("SUM(\"amount\")"));
    assert!(sql.contains("AS \"rank\""));
    assert!(sql.contains("AS \"user_total\""));
}

#[test]
fn test_window_with_select_columns() {
    let qb = QueryBuilder::new("orders")
        .unwrap()
        .select(vec!["id".to_string(), "amount".to_string()])
        .unwrap()
        .window(
            WindowFunction::RowNumber,
            WindowSpec::new().order_by("amount", OrderDirection::Desc),
            "rank",
        )
        .unwrap();
    let (sql, _) = qb.build_select();
    assert!(sql.contains("\"id\", \"amount\""));
    assert!(sql.contains("ROW_NUMBER() OVER"));
}

#[test]
fn test_window_ntile() {
    let qb = QueryBuilder::new("orders")
        .unwrap()
        .window(
            WindowFunction::Ntile(4),
            WindowSpec::new().order_by("amount", OrderDirection::Desc),
            "quartile",
        )
        .unwrap();
    let (sql, _) = qb.build_select();
    assert!(sql.contains("NTILE(4) OVER"));
    assert!(sql.contains("AS \"quartile\""));
}

#[test]
fn test_window_first_last_value() {
    let qb = QueryBuilder::new("orders")
        .unwrap()
        .window(
            WindowFunction::FirstValue("amount".to_string()),
            WindowSpec::new()
                .partition_by(&["user_id"])
                .order_by("created_at", OrderDirection::Asc),
            "first_amount",
        )
        .unwrap()
        .window(
            WindowFunction::LastValue("amount".to_string()),
            WindowSpec::new()
                .partition_by(&["user_id"])
                .order_by("created_at", OrderDirection::Asc),
            "last_amount",
        )
        .unwrap();
    let (sql, _) = qb.build_select();
    assert!(sql.contains("FIRST_VALUE(\"amount\")"));
    assert!(sql.contains("LAST_VALUE(\"amount\")"));
}

#[test]
fn debug_window_sql() {
    // This test demonstrates the generated SQL for window functions
    let qb = QueryBuilder::new("orders")
        .unwrap()
        .select(vec!["id".to_string(), "user_id".to_string(), "amount".to_string()])
        .unwrap()
        .window(
            WindowFunction::RowNumber,
            WindowSpec::new().order_by("amount", OrderDirection::Desc),
            "rank",
        )
        .unwrap()
        .window(
            WindowFunction::Sum("amount".to_string()),
            WindowSpec::new()
                .partition_by(&["user_id"])
                .order_by("created_at", OrderDirection::Asc),
            "running_total",
        )
        .unwrap();
    let (sql, _) = qb.build_select();
    eprintln!("Generated Window SQL:\n{}", sql);

    // Verify the expected SQL structure
    assert!(sql.contains("SELECT \"id\", \"user_id\", \"amount\""));
    assert!(sql.contains("ROW_NUMBER() OVER (ORDER BY \"amount\" DESC) AS \"rank\""));
    assert!(sql.contains("SUM(\"amount\") OVER (PARTITION BY \"user_id\" ORDER BY \"created_at\" ASC) AS \"running_total\""));
}

#[test]
fn test_union_basic() {
    let q1 = QueryBuilder::new("active_users").unwrap()
        .select(vec!["id".to_string(), "name".to_string()]).unwrap();
    let q2 = QueryBuilder::new("archived_users").unwrap()
        .select(vec!["id".to_string(), "name".to_string()]).unwrap();

    let combined = q1.union(q2);
    let (sql, _) = combined.build_select();
    assert!(sql.contains(" UNION "));
    assert!(sql.contains("\"active_users\""));
    assert!(sql.contains("\"archived_users\""));
}

#[test]
fn test_union_all() {
    let q1 = QueryBuilder::new("orders").unwrap();
    let q2 = QueryBuilder::new("returns").unwrap();

    let combined = q1.union_all(q2);
    let (sql, _) = combined.build_select();
    assert!(sql.contains(" UNION ALL "));
}

#[test]
fn test_intersect() {
    let q1 = QueryBuilder::new("all_orders").unwrap()
        .select(vec!["id".to_string()]).unwrap();
    let q2 = QueryBuilder::new("paid_orders").unwrap()
        .select(vec!["id".to_string()]).unwrap();

    let combined = q1.intersect(q2);
    let (sql, _) = combined.build_select();
    assert!(sql.contains(" INTERSECT "));
}

#[test]
fn test_except() {
    let q1 = QueryBuilder::new("all_products").unwrap();
    let q2 = QueryBuilder::new("discontinued").unwrap();

    let combined = q1.except(q2);
    let (sql, _) = combined.build_select();
    assert!(sql.contains(" EXCEPT "));
}

#[test]
fn test_union_with_params() {
    let q1 = QueryBuilder::new("users").unwrap()
        .where_clause("active", Operator::Eq, ExtractedValue::Bool(true)).unwrap();
    let q2 = QueryBuilder::new("admins").unwrap()
        .where_clause("user_role", Operator::Eq, ExtractedValue::String("admin".to_string())).unwrap();

    let combined = q1.union(q2);
    let (sql, params) = combined.build_select();
    assert_eq!(params.len(), 2);
    // Check that parameter indices are adjusted
    assert!(sql.contains("$1"));
    assert!(sql.contains("$2"));
}

#[test]
fn test_multiple_unions() {
    let q1 = QueryBuilder::new("table1").unwrap();
    let q2 = QueryBuilder::new("table2").unwrap();
    let q3 = QueryBuilder::new("table3").unwrap();

    let combined = q1.union(q2).union(q3);
    let (sql, _) = combined.build_select();
    // Should have two UNION keywords
    assert_eq!(sql.matches(" UNION ").count(), 2);
}

#[test]
fn test_set_operations_comprehensive() {
    // Test SQL generation for all set operations
    let q1 = QueryBuilder::new("active_users").unwrap()
        .select(vec!["id".to_string(), "name".to_string()]).unwrap();
    let q2 = QueryBuilder::new("archived_users").unwrap()
        .select(vec!["id".to_string(), "name".to_string()]).unwrap();

    let combined = q1.union(q2);
    let (sql, _) = combined.build_select();
    eprintln!("UNION SQL:\n{}", sql);
    assert_eq!(
        sql,
        "SELECT \"id\", \"name\" FROM \"active_users\" UNION SELECT \"id\", \"name\" FROM \"archived_users\""
    );

    // Test INTERSECT ALL
    let q1 = QueryBuilder::new("orders").unwrap();
    let q2 = QueryBuilder::new("paid_orders").unwrap();
    let combined = q1.intersect_all(q2);
    let (sql, _) = combined.build_select();
    eprintln!("INTERSECT ALL SQL:\n{}", sql);
    assert!(sql.contains(" INTERSECT ALL "));

    // Test EXCEPT ALL
    let q1 = QueryBuilder::new("products").unwrap();
    let q2 = QueryBuilder::new("discontinued").unwrap();
    let combined = q1.except_all(q2);
    let (sql, _) = combined.build_select();
    eprintln!("EXCEPT ALL SQL:\n{}", sql);
    assert!(sql.contains(" EXCEPT ALL "));
}

#[test]
fn test_json_contains() {
    let qb = QueryBuilder::new("users").unwrap()
        .where_json_contains("metadata", r#"{"role": "admin"}"#).unwrap();
    let (sql, _) = qb.build_select();
    assert!(sql.contains("@>"), "SQL should contain @> operator");
    assert!(sql.contains("::jsonb"), "SQL should cast to jsonb");
    assert!(sql.contains(r#"'{"role": "admin"}'"#), "SQL should contain JSON value");
}

#[test]
fn test_json_contained_by() {
    let qb = QueryBuilder::new("users").unwrap()
        .where_json_contained_by("metadata", r#"{"premium": true}"#).unwrap();
    let (sql, _) = qb.build_select();
    assert!(sql.contains("<@"), "SQL should contain <@ operator");
    assert!(sql.contains("::jsonb"), "SQL should cast to jsonb");
}

#[test]
fn test_json_key_exists() {
    let qb = QueryBuilder::new("users").unwrap()
        .where_json_key_exists("metadata", "email").unwrap();
    let (sql, params) = qb.build_select();
    assert!(sql.contains("?"), "SQL should contain ? operator");
    assert_eq!(params.len(), 1, "Should have one parameter");
    if let ExtractedValue::String(s) = &params[0] {
        assert_eq!(s, "email");
    } else {
        panic!("Expected String parameter");
    }
}

#[test]
fn test_json_any_key_exists() {
    let qb = QueryBuilder::new("users").unwrap()
        .where_json_any_key_exists("metadata", &["email", "phone"]).unwrap();
    let (sql, _) = qb.build_select();
    assert!(sql.contains("?|"), "SQL should contain ?| operator");
    assert!(sql.contains("ARRAY["), "SQL should contain ARRAY");
    assert!(sql.contains("'email'"), "SQL should contain email key");
    assert!(sql.contains("'phone'"), "SQL should contain phone key");
}

#[test]
fn test_json_all_keys_exist() {
    let qb = QueryBuilder::new("users").unwrap()
        .where_json_all_keys_exist("metadata", &["email", "name"]).unwrap();
    let (sql, _) = qb.build_select();
    assert!(sql.contains("?&"), "SQL should contain ?& operator");
    assert!(sql.contains("ARRAY["), "SQL should contain ARRAY");
    assert!(sql.contains("'email'"), "SQL should contain email key");
    assert!(sql.contains("'name'"), "SQL should contain name key");
}

#[test]
fn test_json_contains_with_quote_escaping() {
    let qb = QueryBuilder::new("users").unwrap()
        .where_json_contains("metadata", r#"{"name": "O'Brien"}"#).unwrap();
    let (sql, _) = qb.build_select();
    // Single quotes should be escaped
    assert!(sql.contains("O''Brien"), "Single quotes should be escaped");
}

#[test]
fn test_multiple_json_conditions() {
    let qb = QueryBuilder::new("users").unwrap()
        .where_json_contains("metadata", r#"{"role": "admin"}"#).unwrap()
        .where_json_key_exists("metadata", "email").unwrap();
    let (sql, params) = qb.build_select();
    assert!(sql.contains("@>"), "SQL should contain @> operator");
    assert!(sql.contains("?"), "SQL should contain ? operator");
    assert!(sql.contains("AND"), "SQL should have AND between conditions");
    assert_eq!(params.len(), 1, "Should have one parameter for key exists");
}

#[test]
fn debug_json_sql() {
    // This test demonstrates the generated SQL for JSONB operations
    eprintln!("\n=== JSONB Operator SQL Generation ===\n");

    // Test 1: JSON contains
    let qb = QueryBuilder::new("users").unwrap()
        .where_json_contains("metadata", r#"{"role": "admin"}"#).unwrap();
    let (sql, params) = qb.build_select();
    eprintln!("1. JSON Contains (@>):");
    eprintln!("   SQL: {}", sql);
    eprintln!("   Params: {:?}\n", params);

    // Test 2: JSON contained by
    let qb = QueryBuilder::new("users").unwrap()
        .where_json_contained_by("settings", r#"{"theme": "dark"}"#).unwrap();
    let (sql, params) = qb.build_select();
    eprintln!("2. JSON Contained By (<@):");
    eprintln!("   SQL: {}", sql);
    eprintln!("   Params: {:?}\n", params);

    // Test 3: JSON key exists
    let qb = QueryBuilder::new("users").unwrap()
        .where_json_key_exists("metadata", "email").unwrap();
    let (sql, params) = qb.build_select();
    eprintln!("3. JSON Key Exists (?):");
    eprintln!("   SQL: {}", sql);
    eprintln!("   Params: {:?}\n", params);

    // Test 4: JSON any key exists
    let qb = QueryBuilder::new("users").unwrap()
        .where_json_any_key_exists("metadata", &["email", "phone"]).unwrap();
    let (sql, params) = qb.build_select();
    eprintln!("4. JSON Any Key Exists (?|):");
    eprintln!("   SQL: {}", sql);
    eprintln!("   Params: {:?}\n", params);

    // Test 5: JSON all keys exist
    let qb = QueryBuilder::new("users").unwrap()
        .where_json_all_keys_exist("metadata", &["email", "name", "age"]).unwrap();
    let (sql, params) = qb.build_select();
    eprintln!("5. JSON All Keys Exist (?&):");
    eprintln!("   SQL: {}", sql);
    eprintln!("   Params: {:?}\n", params);

    // Test 6: Multiple conditions
    let qb = QueryBuilder::new("users").unwrap()
        .where_json_contains("metadata", r#"{"role": "admin"}"#).unwrap()
        .where_json_key_exists("metadata", "verified").unwrap()
        .where_json_all_keys_exist("profile", &["name", "email"]).unwrap();
    let (sql, params) = qb.build_select();
    eprintln!("6. Multiple JSONB Conditions:");
    eprintln!("   SQL: {}", sql);
    eprintln!("   Params: {:?}\n", params);

    eprintln!("=== End of JSONB SQL Examples ===\n");
}

#[test]
fn test_update_returning_specific_columns() {
    let qb = QueryBuilder::new("users").unwrap()
        .where_clause("id", Operator::Eq, ExtractedValue::Int(1)).unwrap()
        .returning(&["id", "name", "updated_at"]).unwrap();

    let (sql, _) = qb.build_update(&[("status".to_string(), ExtractedValue::String("inactive".to_string()))]).unwrap();
    assert!(sql.contains("RETURNING"));
    assert!(sql.contains("\"id\""));
    assert!(sql.contains("\"name\""));
    assert!(sql.contains("\"updated_at\""));
    assert_eq!(sql, "UPDATE \"users\" SET \"status\" = $1 WHERE \"id\" = $2 RETURNING \"id\", \"name\", \"updated_at\"");
}

#[test]
fn test_update_returning_all() {
    let qb = QueryBuilder::new("users").unwrap()
        .where_clause("id", Operator::Eq, ExtractedValue::Int(1)).unwrap()
        .returning_all();

    let (sql, _) = qb.build_update(&[("status".to_string(), ExtractedValue::String("inactive".to_string()))]).unwrap();
    assert!(sql.contains("RETURNING *"));
    assert_eq!(sql, "UPDATE \"users\" SET \"status\" = $1 WHERE \"id\" = $2 RETURNING *");
}

#[test]
fn test_delete_returning() {
    let qb = QueryBuilder::new("users").unwrap()
        .where_clause("id", Operator::Eq, ExtractedValue::Int(1)).unwrap()
        .returning(&["id", "email"]).unwrap();

    let (sql, _) = qb.build_delete();
    assert!(sql.contains("DELETE FROM"));
    assert!(sql.contains("RETURNING"));
    assert!(sql.contains("\"id\""));
    assert!(sql.contains("\"email\""));
    assert_eq!(sql, "DELETE FROM \"users\" WHERE \"id\" = $1 RETURNING \"id\", \"email\"");
}

#[test]
fn test_delete_returning_all() {
    let qb = QueryBuilder::new("users").unwrap()
        .where_clause("id", Operator::Eq, ExtractedValue::Int(1)).unwrap()
        .returning_all();

    let (sql, _) = qb.build_delete();
    assert!(sql.contains("RETURNING *"));
    assert_eq!(sql, "DELETE FROM \"users\" WHERE \"id\" = $1 RETURNING *");
}

#[test]
fn test_returning_validates_identifiers() {
    let result = QueryBuilder::new("users").unwrap()
        .returning(&["id; DROP TABLE users"]);
    assert!(result.is_err());
}

#[test]
fn test_returning_allows_asterisk() {
    let result = QueryBuilder::new("users").unwrap()
        .returning(&["*"]);
    assert!(result.is_ok());
    let qb = result.unwrap();
    let (sql, _) = qb.build_delete();
    assert!(sql.contains("RETURNING *"));
}

#[test]
fn test_clear_returning() {
    let qb = QueryBuilder::new("users").unwrap()
        .where_clause("id", Operator::Eq, ExtractedValue::Int(1)).unwrap()
        .returning(&["id", "name"]).unwrap()
        .clear_returning();

    let (sql, _) = qb.build_update(&[("status".to_string(), ExtractedValue::String("inactive".to_string()))]).unwrap();
    assert!(!sql.contains("RETURNING"));
    assert_eq!(sql, "UPDATE \"users\" SET \"status\" = $1 WHERE \"id\" = $2");
}

#[test]
fn test_update_without_returning() {
    let qb = QueryBuilder::new("users").unwrap()
        .where_clause("id", Operator::Eq, ExtractedValue::Int(1)).unwrap();

    let (sql, _) = qb.build_update(&[("status".to_string(), ExtractedValue::String("inactive".to_string()))]).unwrap();
    assert!(!sql.contains("RETURNING"));
}

#[test]
fn test_delete_without_returning() {
    let qb = QueryBuilder::new("users").unwrap()
        .where_clause("id", Operator::Eq, ExtractedValue::Int(1)).unwrap();

    let (sql, _) = qb.build_delete();
    assert!(!sql.contains("RETURNING"));
}

// =============================================================================
// Convenience Method Tests
// =============================================================================

#[test]
fn test_count_agg_convenience() {
    let qb = QueryBuilder::new("orders").unwrap()
        .count_agg(Some("total_count")).unwrap()
        .group_by(&["status"]).unwrap();
    let (sql, _) = qb.build_select();
    assert!(sql.contains("COUNT(*)"));
    assert!(sql.contains("AS \"total_count\""));
    assert!(sql.contains("GROUP BY \"status\""));
}

#[test]
fn test_count_distinct_convenience() {
    let qb = QueryBuilder::new("events").unwrap()
        .count_distinct("user_id", Some("unique_users")).unwrap();
    let (sql, _) = qb.build_select();
    assert!(sql.contains("COUNT(DISTINCT \"user_id\")"));
    assert!(sql.contains("AS \"unique_users\""));
}

#[test]
fn test_sum_convenience() {
    let qb = QueryBuilder::new("orders").unwrap()
        .sum("amount", Some("total_amount")).unwrap()
        .group_by(&["user_id"]).unwrap();
    let (sql, _) = qb.build_select();
    assert!(sql.contains("SUM(\"amount\")"));
    assert!(sql.contains("AS \"total_amount\""));
}

#[test]
fn test_avg_convenience() {
    let qb = QueryBuilder::new("orders").unwrap()
        .avg("amount", Some("avg_amount")).unwrap();
    let (sql, _) = qb.build_select();
    assert!(sql.contains("AVG(\"amount\")"));
}

#[test]
fn test_min_max_convenience() {
    let qb = QueryBuilder::new("products").unwrap()
        .min("price", Some("min_price")).unwrap()
        .max("price", Some("max_price")).unwrap();
    let (sql, _) = qb.build_select();
    assert!(sql.contains("MIN(\"price\")"));
    assert!(sql.contains("MAX(\"price\")"));
}

#[test]
fn test_having_count_convenience() {
    let qb = QueryBuilder::new("orders").unwrap()
        .count_agg(Some("order_count")).unwrap()
        .group_by(&["user_id"]).unwrap()
        .having_count(Operator::Gt, ExtractedValue::Int(5)).unwrap();
    let (sql, _) = qb.build_select();
    assert!(sql.contains("HAVING COUNT(*) > $1"));
}

#[test]
fn test_having_sum_convenience() {
    let qb = QueryBuilder::new("orders").unwrap()
        .sum("amount", Some("total")).unwrap()
        .group_by(&["user_id"]).unwrap()
        .having_sum("amount", Operator::Gt, ExtractedValue::Int(1000)).unwrap();
    let (sql, _) = qb.build_select();
    assert!(sql.contains("HAVING SUM(\"amount\") > $1"));
}

#[test]
fn test_row_number_convenience() {
    let spec = WindowSpec::new()
        .partition_by(&["user_id"])
        .order_by("created_at", OrderDirection::Desc);
    let qb = QueryBuilder::new("orders").unwrap()
        .row_number(spec, "rank").unwrap();
    let (sql, _) = qb.build_select();
    assert!(sql.contains("ROW_NUMBER()"));
    assert!(sql.contains("PARTITION BY \"user_id\""));
    assert!(sql.contains("ORDER BY \"created_at\" DESC"));
    assert!(sql.contains("AS \"rank\""));
}

#[test]
fn test_rank_convenience() {
    let spec = WindowSpec::new()
        .order_by("score", OrderDirection::Desc);
    let qb = QueryBuilder::new("students").unwrap()
        .rank(spec, "student_rank").unwrap();
    let (sql, _) = qb.build_select();
    assert!(sql.contains("RANK()"));
    assert!(sql.contains("AS \"student_rank\""));
}

#[test]
fn test_lag_convenience() {
    let spec = WindowSpec::new()
        .order_by("timestamp", OrderDirection::Asc);
    let qb = QueryBuilder::new("stock_prices").unwrap()
        .lag("price", Some(1), Some(ExtractedValue::Int(0)), spec, "prev_price").unwrap();
    let (sql, _) = qb.build_select();
    assert!(sql.contains("LAG(\"price\""));
    assert!(sql.contains("AS \"prev_price\""));
}

#[test]
fn test_lead_convenience() {
    let spec = WindowSpec::new()
        .order_by("timestamp", OrderDirection::Asc);
    let qb = QueryBuilder::new("stock_prices").unwrap()
        .lead("price", Some(1), None, spec, "next_price").unwrap();
    let (sql, _) = qb.build_select();
    assert!(sql.contains("LEAD(\"price\""));
    assert!(sql.contains("AS \"next_price\""));
}

#[test]
fn test_window_sum_convenience() {
    let spec = WindowSpec::new()
        .partition_by(&["category"]);
    let qb = QueryBuilder::new("sales").unwrap()
        .window_sum("amount", spec, "running_total").unwrap();
    let (sql, _) = qb.build_select();
    assert!(sql.contains("SUM(\"amount\")"));
    assert!(sql.contains("OVER"));
    assert!(sql.contains("PARTITION BY \"category\""));
}

#[test]
fn test_from_cte_convenience() {
    let subquery = QueryBuilder::new("orders").unwrap()
        .where_clause("total", Operator::Gt, ExtractedValue::Int(1000)).unwrap();

    let qb = QueryBuilder::from_cte("high_value_orders", subquery).unwrap()
        .order_by("total", OrderDirection::Desc).unwrap()
        .limit(10);

    let (sql, params) = qb.build_select();
    assert!(sql.starts_with("WITH \"high_value_orders\" AS"));
    assert!(sql.contains("SELECT * FROM \"high_value_orders\""));
    assert!(sql.contains("ORDER BY \"total\" DESC"));
    assert!(sql.contains("LIMIT"));
    assert_eq!(params.len(), 2); // $1 for 1000, $2 for limit 10
}

#[test]
fn test_combined_convenience_methods() {
    // Test a complex query using multiple convenience methods
    let qb = QueryBuilder::new("orders").unwrap()
        .sum("amount", Some("total")).unwrap()
        .avg("amount", Some("average")).unwrap()
        .count_agg(Some("count")).unwrap()
        .group_by(&["user_id"]).unwrap()
        .having_sum("amount", Operator::Gt, ExtractedValue::Int(1000)).unwrap()
        .order_by("total", OrderDirection::Desc).unwrap()
        .limit(10);

    let (sql, _) = qb.build_select();
    assert!(sql.contains("SUM(\"amount\") AS \"total\""));
    assert!(sql.contains("AVG(\"amount\") AS \"average\""));
    assert!(sql.contains("COUNT(*) AS \"count\""));
    assert!(sql.contains("GROUP BY \"user_id\""));
    assert!(sql.contains("HAVING SUM(\"amount\") >"));
    assert!(sql.contains("ORDER BY \"total\" DESC"));
}
