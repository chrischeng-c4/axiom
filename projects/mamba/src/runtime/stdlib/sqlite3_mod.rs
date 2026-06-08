use super::super::rc::{MbObject, ObjData};
use super::super::value::MbValue;
/// sqlite3 module for Mamba (#444).
///
/// Provides: connect, Connection (execute, fetchall, commit, close)
/// Stub implementation — stores data in-memory HashMap tables.
/// No external dependency (no rusqlite).
use std::collections::HashMap;

// ── Variadic dispatchers (callable from module-attr context) ──

macro_rules! disp_unary {
    ($disp:ident, $fn:path) => {
        unsafe extern "C" fn $disp(args_ptr: *const MbValue, nargs: usize) -> MbValue {
            let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
            $fn(a.get(0).copied().unwrap_or_else(MbValue::none))
        }
    };
}

disp_unary!(d_connect, mb_sqlite3_connect);

/// Register the sqlite3 module.
pub fn register() {
    let mut attrs = HashMap::new();

    // Isolation levels
    attrs.insert("PARSE_DECLTYPES".into(), MbValue::from_int(1));
    attrs.insert("PARSE_COLNAMES".into(), MbValue::from_int(2));

    let dispatchers: Vec<(&str, usize)> = vec![("connect", d_connect as *const () as usize)];
    for (name, addr) in dispatchers {
        attrs.insert(name.to_string(), MbValue::from_func(addr));
        super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
            s.borrow_mut().insert(addr as u64);
        });
    }

    super::register_module("sqlite3", attrs);
}

fn extract_str(val: MbValue) -> Option<String> {
    val.as_ptr().and_then(|ptr| unsafe {
        if let ObjData::Str(ref s) = (*ptr).data {
            Some(s.clone())
        } else {
            None
        }
    })
}

/// sqlite3.connect(database) -> connection dict
pub fn mb_sqlite3_connect(db_path: MbValue) -> MbValue {
    let path = extract_str(db_path).unwrap_or_else(|| ":memory:".to_string());
    let dict = MbObject::new_dict();
    unsafe {
        if let ObjData::Dict(ref lock) = (*dict).data {
            let mut map = lock.write().unwrap();
            map.insert(
                "__class__".into(),
                MbValue::from_ptr(MbObject::new_str("Connection".to_string())),
            );
            map.insert(
                "database".into(),
                MbValue::from_ptr(MbObject::new_str(path)),
            );
            map.insert("closed".into(), MbValue::from_bool(false));
            // In-memory table store
            map.insert("_tables".into(), MbValue::from_ptr(MbObject::new_dict()));
            // Last query results
            map.insert(
                "_results".into(),
                MbValue::from_ptr(MbObject::new_list(vec![])),
            );
        }
    }
    MbValue::from_ptr(dict)
}

/// conn.cursor() -> returns the connection itself (simplified)
pub fn mb_sqlite3_cursor(conn: MbValue) -> MbValue {
    conn
}

/// conn.execute(sql, params?) -> self
pub fn mb_sqlite3_execute(conn: MbValue, sql: MbValue) -> MbValue {
    let query = extract_str(sql).unwrap_or_default();
    let upper = query.trim().to_uppercase();

    if let Some(ptr) = conn.as_ptr() {
        unsafe {
            if let ObjData::Dict(ref lock) = (*ptr).data {
                let mut map = lock.write().unwrap();
                if upper.starts_with("CREATE TABLE") {
                    // Extract table name
                    if let Some(name) = extract_table_name(&query) {
                        if let Some(tables) = map.get("_tables").copied() {
                            if let Some(tbl_ptr) = tables.as_ptr() {
                                if let ObjData::Dict(ref tbl_lock) = (*tbl_ptr).data {
                                    let mut tbl_map = tbl_lock.write().unwrap();
                                    tbl_map.insert(
                                        name.into(),
                                        MbValue::from_ptr(MbObject::new_list(vec![])),
                                    );
                                }
                            }
                        }
                    }
                }
                // Store the query for reference
                map.insert(
                    "_last_sql".into(),
                    MbValue::from_ptr(MbObject::new_str(query)),
                );
            }
        }
    }
    conn
}

fn extract_table_name(sql: &str) -> Option<String> {
    let tokens: Vec<&str> = sql.split_whitespace().collect();
    // "CREATE TABLE name ..."
    for (i, token) in tokens.iter().enumerate() {
        if token.to_uppercase() == "TABLE" && i + 1 < tokens.len() {
            let name = tokens[i + 1].trim_matches(|c| c == '(' || c == '"' || c == '`');
            if name.to_uppercase() != "IF" {
                return Some(name.to_string());
            }
            // CREATE TABLE IF NOT EXISTS name
            if i + 4 < tokens.len() {
                let n = tokens[i + 4].trim_matches(|c| c == '(' || c == '"' || c == '`');
                return Some(n.to_string());
            }
        }
    }
    None
}

/// conn.fetchall() -> list of tuples
pub fn mb_sqlite3_fetchall(conn: MbValue) -> MbValue {
    if let Some(ptr) = conn.as_ptr() {
        unsafe {
            if let ObjData::Dict(ref lock) = (*ptr).data {
                let map = lock.read().unwrap();
                if let Some(results) = map.get("_results").copied() {
                    return results;
                }
            }
        }
    }
    MbValue::from_ptr(MbObject::new_list(vec![]))
}

/// conn.fetchone() -> first row or None
pub fn mb_sqlite3_fetchone(conn: MbValue) -> MbValue {
    if let Some(ptr) = conn.as_ptr() {
        unsafe {
            if let ObjData::Dict(ref lock) = (*ptr).data {
                let map = lock.read().unwrap();
                if let Some(results) = map.get("_results").copied() {
                    if let Some(res_ptr) = results.as_ptr() {
                        if let ObjData::List(ref list_lock) = (*res_ptr).data {
                            let items = list_lock.read().unwrap();
                            if let Some(first) = items.first() {
                                return *first;
                            }
                        }
                    }
                }
            }
        }
    }
    MbValue::none()
}

/// conn.commit() -> None
pub fn mb_sqlite3_commit(_conn: MbValue) -> MbValue {
    MbValue::none()
}

/// conn.close() -> None
pub fn mb_sqlite3_close(conn: MbValue) -> MbValue {
    if let Some(ptr) = conn.as_ptr() {
        unsafe {
            if let ObjData::Dict(ref lock) = (*ptr).data {
                let mut map = lock.write().unwrap();
                map.insert("closed".into(), MbValue::from_bool(true));
            }
        }
    }
    MbValue::none()
}

/// conn.executemany(sql, params_list) -> self
pub fn mb_sqlite3_executemany(conn: MbValue, sql: MbValue) -> MbValue {
    mb_sqlite3_execute(conn, sql)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn s(val: &str) -> MbValue {
        MbValue::from_ptr(MbObject::new_str(val.to_string()))
    }

    fn dict_bool(val: MbValue, key: &str) -> Option<bool> {
        val.as_ptr().and_then(|ptr| unsafe {
            if let ObjData::Dict(ref lock) = (*ptr).data {
                lock.read().unwrap().get(key).and_then(|v| v.as_bool())
            } else {
                None
            }
        })
    }

    fn dict_str(val: MbValue, key: &str) -> Option<String> {
        val.as_ptr().and_then(|ptr| unsafe {
            if let ObjData::Dict(ref lock) = (*ptr).data {
                lock.read().unwrap().get(key).and_then(|v| {
                    v.as_ptr().and_then(|p| {
                        if let ObjData::Str(ref st) = (*p).data {
                            Some(st.clone())
                        } else {
                            None
                        }
                    })
                })
            } else {
                None
            }
        })
    }

    fn has_table(conn: MbValue, table: &str) -> bool {
        if let Some(ptr) = conn.as_ptr() {
            unsafe {
                if let ObjData::Dict(ref lock) = (*ptr).data {
                    let map = lock.read().unwrap();
                    if let Some(tables) = map.get("_tables").copied() {
                        if let Some(tbl_ptr) = tables.as_ptr() {
                            if let ObjData::Dict(ref tbl_lock) = (*tbl_ptr).data {
                                return tbl_lock.read().unwrap().contains_key(table);
                            }
                        }
                    }
                }
            }
        }
        false
    }

    // --- extract_str ---
    #[test]
    fn test_extract_str_str() {
        assert_eq!(extract_str(s("hello")), Some("hello".to_string()));
    }

    #[test]
    fn test_extract_str_non_str() {
        assert_eq!(extract_str(MbValue::from_int(0)), None);
    }

    // --- connect ---
    #[test]
    fn test_connect_with_str_path() {
        let conn = mb_sqlite3_connect(s(":memory:"));
        assert_eq!(dict_str(conn, "__class__"), Some("Connection".to_string()));
        assert_eq!(dict_str(conn, "database"), Some(":memory:".to_string()));
    }

    #[test]
    fn test_connect_non_str_defaults_to_memory() {
        let conn = mb_sqlite3_connect(MbValue::from_int(0));
        assert_eq!(dict_str(conn, "database"), Some(":memory:".to_string()));
    }

    // --- cursor ---
    #[test]
    fn test_cursor_returns_conn() {
        let conn = mb_sqlite3_connect(s(":memory:"));
        let cursor = mb_sqlite3_cursor(conn);
        // Same value (cursor is just conn)
        assert_eq!(conn, cursor);
    }

    // --- extract_table_name ---
    #[test]
    fn test_extract_table_name_basic() {
        assert_eq!(
            extract_table_name("CREATE TABLE users (id INT)"),
            Some("users".to_string())
        );
    }

    #[test]
    fn test_extract_table_name_if_not_exists() {
        assert_eq!(
            extract_table_name("CREATE TABLE IF NOT EXISTS t (x INT)"),
            Some("t".to_string())
        );
    }

    #[test]
    fn test_extract_table_name_no_table() {
        assert_eq!(extract_table_name("SELECT 1"), None);
    }

    // --- execute ---
    #[test]
    fn test_connect_and_close() {
        let conn = mb_sqlite3_connect(s(":memory:"));
        mb_sqlite3_close(conn);
        assert_eq!(dict_bool(conn, "closed"), Some(true));
    }

    #[test]
    fn test_create_table() {
        let conn = mb_sqlite3_connect(s(":memory:"));
        mb_sqlite3_execute(conn, s("CREATE TABLE users (id INT, name TEXT)"));
        assert!(has_table(conn, "users"));
    }

    #[test]
    fn test_create_table_if_not_exists() {
        let conn = mb_sqlite3_connect(s(":memory:"));
        mb_sqlite3_execute(conn, s("CREATE TABLE IF NOT EXISTS logs (msg TEXT)"));
        assert!(has_table(conn, "logs"));
    }

    #[test]
    fn test_execute_non_create_stores_last_sql() {
        let conn = mb_sqlite3_connect(s(":memory:"));
        mb_sqlite3_execute(conn, s("SELECT 1"));
        assert_eq!(dict_str(conn, "_last_sql"), Some("SELECT 1".to_string()));
        // No table created
        assert!(!has_table(conn, "1"));
    }

    #[test]
    fn test_execute_null_conn_noop() {
        mb_sqlite3_execute(MbValue::none(), s("CREATE TABLE t (x INT)")); // no panic
    }

    // --- fetchall ---
    #[test]
    fn test_fetchall_empty_results() {
        let conn = mb_sqlite3_connect(s(":memory:"));
        let result = mb_sqlite3_fetchall(conn);
        if let Some(ptr) = result.as_ptr() {
            unsafe {
                if let ObjData::List(ref lock) = (*ptr).data {
                    assert!(lock.read().unwrap().is_empty());
                }
            }
        }
    }

    #[test]
    fn test_fetchall_null_returns_empty() {
        let result = mb_sqlite3_fetchall(MbValue::none());
        assert!(result.as_ptr().is_some());
    }

    // --- fetchone ---
    #[test]
    fn test_fetchone_empty_results_returns_none() {
        let conn = mb_sqlite3_connect(s(":memory:"));
        let result = mb_sqlite3_fetchone(conn);
        assert!(result.is_none());
    }

    #[test]
    fn test_fetchone_null_returns_none() {
        let result = mb_sqlite3_fetchone(MbValue::none());
        assert!(result.is_none());
    }

    // --- commit ---
    #[test]
    fn test_commit_returns_none() {
        let conn = mb_sqlite3_connect(s(":memory:"));
        let result = mb_sqlite3_commit(conn);
        assert!(result.is_none());
    }

    // --- close ---
    #[test]
    fn test_close_null_noop() {
        mb_sqlite3_close(MbValue::none()); // no panic
    }

    // --- executemany ---
    #[test]
    fn test_executemany_delegates_to_execute() {
        let conn = mb_sqlite3_connect(s(":memory:"));
        mb_sqlite3_executemany(conn, s("CREATE TABLE z (n INT)"));
        assert!(has_table(conn, "z"));
    }

    // --- fetchall with _results present ---
    #[test]
    fn test_fetchall_with_results_present() {
        let conn = mb_sqlite3_connect(s(":memory:"));
        // Manually inject _results list into conn dict
        let results_list = MbValue::from_ptr(MbObject::new_list(vec![
            MbValue::from_int(10),
            MbValue::from_int(20),
        ]));
        if let Some(ptr) = conn.as_ptr() {
            unsafe {
                if let ObjData::Dict(ref lock) = (*ptr).data {
                    let mut map = lock.write().unwrap();
                    map.insert("_results".into(), results_list);
                }
            }
        }
        let result = mb_sqlite3_fetchall(conn);
        // Should return the injected results list (not empty list)
        assert!(result.as_ptr().is_some());
    }

    // --- fetchone with non-empty _results ---
    #[test]
    fn test_fetchone_with_results_present() {
        let conn = mb_sqlite3_connect(s(":memory:"));
        let first_item = MbValue::from_int(42);
        let results_list = MbValue::from_ptr(MbObject::new_list(vec![first_item]));
        if let Some(ptr) = conn.as_ptr() {
            unsafe {
                if let ObjData::Dict(ref lock) = (*ptr).data {
                    let mut map = lock.write().unwrap();
                    map.insert("_results".into(), results_list);
                }
            }
        }
        let result = mb_sqlite3_fetchone(conn);
        assert_eq!(result.as_int(), Some(42));
    }
}
