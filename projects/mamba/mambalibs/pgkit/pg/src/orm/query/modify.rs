//! INSERT, UPDATE, DELETE query building methods for QueryBuilder.

use crate::{DataBridgeError, ExtractedValue, Result};
use super::builder::QueryBuilder;
use super::helpers::{quote_identifier, adjust_param_indices};
use super::types::Operator;

impl QueryBuilder {
    /// Add columns to the RETURNING clause for INSERT/UPDATE/DELETE queries
    pub fn returning(mut self, columns: &[&str]) -> Result<Self> {
        for col in columns {
            if *col != "*" {
                Self::validate_identifier(col)?;
            }
            self.returning.push(col.to_string());
        }
        Ok(self)
    }

    /// Return all columns from INSERT/UPDATE/DELETE
    pub fn returning_all(mut self) -> Self {
        self.returning.push("*".to_string());
        self
    }

    /// Clear RETURNING clause
    pub fn clear_returning(mut self) -> Self {
        self.returning.clear();
        self
    }

    /// Builds an INSERT SQL query string with parameter placeholders.
    ///
    /// Returns the SQL string with $1, $2, etc. placeholders and the parameter values.
    pub fn build_insert(&self, values: &[(String, ExtractedValue)]) -> Result<(String, Vec<ExtractedValue>)> {
        if values.is_empty() {
            return Err(DataBridgeError::Query("Cannot insert with no values".to_string()));
        }

        // Validate column names
        for (col, _) in values {
            Self::validate_identifier(col)?;
        }

        let mut sql = format!("INSERT INTO {} (", quote_identifier(&self.table));
        let columns: Vec<String> = values.iter().map(|(col, _)| quote_identifier(col)).collect();
        sql.push_str(&columns.join(", "));
        sql.push_str(") VALUES (");

        let placeholders: Vec<String> = (1..=values.len()).map(|i| format!("${}", i)).collect();
        sql.push_str(&placeholders.join(", "));
        sql.push(')');
        self.push_returning_clause(&mut sql);

        let params: Vec<ExtractedValue> = values.iter().map(|(_, val)| val.clone()).collect();

        Ok((sql, params))
    }

    /// Builds an UPDATE SQL query string with parameter placeholders.
    ///
    /// Returns the SQL string with $1, $2, etc. placeholders and the parameter values.
    pub fn build_update(&self, values: &[(String, ExtractedValue)]) -> Result<(String, Vec<ExtractedValue>)> {
        if values.is_empty() {
            return Err(DataBridgeError::Query("Cannot update with no values".to_string()));
        }

        // Validate column names
        for (col, _) in values {
            Self::validate_identifier(col)?;
        }

        let mut sql = format!("UPDATE {} SET ", quote_identifier(&self.table));
        let mut params: Vec<ExtractedValue> = Vec::new();

        // SET clause
        let set_parts: Vec<String> = values.iter().map(|(col, val)| {
            params.push(val.clone());
            format!("{} = ${}", quote_identifier(col), params.len())
        }).collect();
        sql.push_str(&set_parts.join(", "));

        // WHERE clause
        if !self.where_conditions.is_empty() {
            sql.push_str(" WHERE ");
            let mut where_parts: Vec<String> = Vec::new();

            for cond in &self.where_conditions {
                let part = self.build_modify_where_condition(cond, &mut params);
                where_parts.push(part);
            }

            sql.push_str(&where_parts.join(" AND "));
        }

        self.push_optional_returning_clause(&mut sql);

        Ok((sql, params))
    }

    /// Builds an UPSERT SQL query (INSERT ON CONFLICT UPDATE).
    pub fn build_upsert(
        &self,
        values: &[(String, ExtractedValue)],
        conflict_target: &[String],
        update_columns: Option<&[String]>,
    ) -> Result<(String, Vec<ExtractedValue>)> {
        // Validation
        if values.is_empty() {
            return Err(DataBridgeError::Query("Cannot upsert with no values".to_string()));
        }
        if conflict_target.is_empty() {
            return Err(DataBridgeError::Query("Conflict target cannot be empty".to_string()));
        }

        // Validate column names
        for (col, _) in values {
            Self::validate_identifier(col)?;
        }
        for col in conflict_target {
            Self::validate_identifier(col)?;
        }
        if let Some(cols) = update_columns {
            for col in cols {
                Self::validate_identifier(col)?;
            }
        }

        // Build INSERT clause
        let mut sql = format!("INSERT INTO {} (", quote_identifier(&self.table));
        let columns: Vec<String> = values.iter().map(|(col, _)| quote_identifier(col)).collect();
        sql.push_str(&columns.join(", "));
        sql.push_str(") VALUES (");

        let placeholders: Vec<String> = (1..=values.len()).map(|i| format!("${}", i)).collect();
        sql.push_str(&placeholders.join(", "));
        sql.push(')');

        // Build ON CONFLICT clause
        sql.push_str(" ON CONFLICT (");
        let quoted_targets: Vec<String> = conflict_target.iter().map(|c| quote_identifier(c)).collect();
        sql.push_str(&quoted_targets.join(", "));
        sql.push_str(") DO UPDATE SET ");

        // Determine which columns to update
        let columns_to_update: Vec<String> = if let Some(update_cols) = update_columns {
            update_cols.to_vec()
        } else {
            values.iter()
                .map(|(col, _)| col.clone())
                .filter(|col| !conflict_target.contains(col))
                .collect()
        };

        if columns_to_update.is_empty() {
            return Err(DataBridgeError::Query(
                "No columns to update after excluding conflict target".to_string()
            ));
        }

        // Build SET clause using EXCLUDED
        let set_parts: Vec<String> = columns_to_update
            .iter()
            .map(|col| format!("{} = EXCLUDED.{}", quote_identifier(col), quote_identifier(col)))
            .collect();
        sql.push_str(&set_parts.join(", "));

        self.push_returning_clause(&mut sql);

        let params: Vec<ExtractedValue> = values.iter().map(|(_, val)| val.clone()).collect();

        Ok((sql, params))
    }

    /// Builds a DELETE SQL query string with parameter placeholders.
    ///
    /// Returns the SQL string with $1, $2, etc. placeholders and the parameter values.
    pub fn build_delete(&self) -> (String, Vec<ExtractedValue>) {
        let mut sql = format!("DELETE FROM {}", quote_identifier(&self.table));
        let mut params: Vec<ExtractedValue> = Vec::new();

        // WHERE clause
        if !self.where_conditions.is_empty() {
            sql.push_str(" WHERE ");
            let mut where_parts: Vec<String> = Vec::new();

            for cond in &self.where_conditions {
                let part = self.build_modify_where_condition(cond, &mut params);
                where_parts.push(part);
            }

            sql.push_str(&where_parts.join(" AND "));
        }

        self.push_optional_returning_clause(&mut sql);

        (sql, params)
    }

    /// Helper to build a single WHERE condition for UPDATE/DELETE
    fn build_modify_where_condition(&self, cond: &super::builder::WhereCondition, params: &mut Vec<ExtractedValue>) -> String {
        match cond.operator {
            Operator::InSubquery => {
                if let Some(ref sq) = cond.subquery {
                    let adjusted_sql = adjust_param_indices(&sq.sql, params.len());
                    params.extend(sq.params.clone());
                    format!("{} IN ({})", quote_identifier(&cond.field), adjusted_sql)
                } else {
                    format!("{} IN (NULL)", quote_identifier(&cond.field))
                }
            }
            Operator::NotInSubquery => {
                if let Some(ref sq) = cond.subquery {
                    let adjusted_sql = adjust_param_indices(&sq.sql, params.len());
                    params.extend(sq.params.clone());
                    format!("{} NOT IN ({})", quote_identifier(&cond.field), adjusted_sql)
                } else {
                    format!("{} NOT IN (NULL)", quote_identifier(&cond.field))
                }
            }
            Operator::Exists => {
                if let Some(ref sq) = cond.subquery {
                    let adjusted_sql = adjust_param_indices(&sq.sql, params.len());
                    params.extend(sq.params.clone());
                    format!("EXISTS ({})", adjusted_sql)
                } else {
                    "EXISTS (NULL)".to_string()
                }
            }
            Operator::NotExists => {
                if let Some(ref sq) = cond.subquery {
                    let adjusted_sql = adjust_param_indices(&sq.sql, params.len());
                    params.extend(sq.params.clone());
                    format!("NOT EXISTS ({})", adjusted_sql)
                } else {
                    "NOT EXISTS (NULL)".to_string()
                }
            }
            Operator::IsNull | Operator::IsNotNull => {
                let quoted_field = quote_identifier(&cond.field);
                format!("{} {}", quoted_field, cond.operator.to_sql())
            }
            Operator::In | Operator::NotIn => {
                let quoted_field = quote_identifier(&cond.field);
                if let Some(ref value) = cond.value {
                    params.push(value.clone());
                    format!("{} {} (${})", quoted_field, cond.operator.to_sql(), params.len())
                } else {
                    format!("{} {} (NULL)", quoted_field, cond.operator.to_sql())
                }
            }
            Operator::JsonContains | Operator::JsonContainedBy => {
                if let Some(ExtractedValue::String(json)) = &cond.value {
                    format!("{} {} '{}'::jsonb",
                        quote_identifier(&cond.field),
                        cond.operator.to_sql(),
                        json.replace("'", "''")
                    )
                } else {
                    format!("{} {} NULL", quote_identifier(&cond.field), cond.operator.to_sql())
                }
            }
            Operator::JsonKeyExists => {
                let quoted_field = quote_identifier(&cond.field);
                if let Some(ref value) = cond.value {
                    params.push(value.clone());
                    format!("{} {} ${}", quoted_field, cond.operator.to_sql(), params.len())
                } else {
                    format!("{} {} NULL", quoted_field, cond.operator.to_sql())
                }
            }
            Operator::JsonAnyKeyExists | Operator::JsonAllKeysExist => {
                if let Some(ExtractedValue::String(arr)) = &cond.value {
                    format!("{} {} {}",
                        quote_identifier(&cond.field),
                        cond.operator.to_sql(),
                        arr
                    )
                } else {
                    format!("{} {} NULL", quote_identifier(&cond.field), cond.operator.to_sql())
                }
            }
            // Array operators
            Operator::Any | Operator::Has => {
                let quoted_field = quote_identifier(&cond.field);
                if let Some(ref value) = cond.value {
                    params.push(value.clone());
                    format!("${} = ANY({})", params.len(), quoted_field)
                } else {
                    format!("NULL = ANY({})", quoted_field)
                }
            }
            Operator::ArrayContains | Operator::HasAll => {
                let quoted_field = quote_identifier(&cond.field);
                if let Some(ref value) = cond.value {
                    params.push(value.clone());
                    format!("{} @> ${}", quoted_field, params.len())
                } else {
                    format!("{} @> NULL", quoted_field)
                }
            }
            Operator::ArrayContainedBy => {
                let quoted_field = quote_identifier(&cond.field);
                if let Some(ref value) = cond.value {
                    params.push(value.clone());
                    format!("{} <@ ${}", quoted_field, params.len())
                } else {
                    format!("{} <@ NULL", quoted_field)
                }
            }
            Operator::ArrayOverlaps | Operator::HasAny => {
                let quoted_field = quote_identifier(&cond.field);
                if let Some(ref value) = cond.value {
                    params.push(value.clone());
                    format!("{} && ${}", quoted_field, params.len())
                } else {
                    format!("{} && NULL", quoted_field)
                }
            }
            _ => {
                let quoted_field = quote_identifier(&cond.field);
                if let Some(ref value) = cond.value {
                    params.push(value.clone());
                    format!("{} {} ${}", quoted_field, cond.operator.to_sql(), params.len())
                } else {
                    format!("{} {} NULL", quoted_field, cond.operator.to_sql())
                }
            }
        }
    }

    fn push_returning_clause(&self, sql: &mut String) {
        sql.push_str(" RETURNING ");
        if self.returning.is_empty() || self.returning.contains(&"*".to_string()) {
            sql.push('*');
        } else {
            let cols: Vec<String> = self.returning.iter()
                .map(|c| quote_identifier(c))
                .collect();
            sql.push_str(&cols.join(", "));
        }
    }

    fn push_optional_returning_clause(&self, sql: &mut String) {
        if !self.returning.is_empty() {
            self.push_returning_clause(sql);
        }
    }
}
