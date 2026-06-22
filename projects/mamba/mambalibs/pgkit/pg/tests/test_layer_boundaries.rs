//! Audit the driver/orm/migrate layer boundary.
//!
//! Phase A (this test): grep every `.rs` file under `src/driver/` and
//! fail if it references `crate::orm::` or `crate::migrate::`. The
//! driver layer must not depend on either higher-level layer.
//!
//! Phase B (future): once each layer is extracted into its own crate,
//! this grep test retires and the real `Cargo.toml` dep graph becomes
//! the enforcement (driver's manifest will simply not list the higher
//! crates as dependencies).

use std::fs;
use std::path::{Path, PathBuf};

const FORBIDDEN_FROM_DRIVER: &[&str] = &[
    "use crate::orm::",
    "use crate::migrate::",
    "crate::orm::",
    "crate::migrate::",
];

#[test]
fn driver_must_not_depend_on_orm_or_migrate() {
    let driver = driver_root();
    assert!(
        driver.is_dir(),
        "expected driver/ at {}: re-check workspace layout",
        driver.display()
    );

    let mut offenders: Vec<(PathBuf, &'static str)> = Vec::new();
    walk(&driver, &mut |path| {
        if path.extension().and_then(|s| s.to_str()) != Some("rs") {
            return;
        }
        let src =
            fs::read_to_string(path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
        for needle in FORBIDDEN_FROM_DRIVER {
            if src.contains(needle) {
                offenders.push((path.to_path_buf(), needle));
            }
        }
    });

    assert!(
        offenders.is_empty(),
        "driver/ layer must not depend on orm/ or migrate/. Offenders:\n{}",
        offenders
            .iter()
            .map(|(p, n)| format!("  - {} contains '{}'", p.display(), n))
            .collect::<Vec<_>>()
            .join("\n")
    );
}

#[test]
fn query_builder_execution_must_delegate_to_driver_connection() {
    let execute = query_execute_source();
    assert!(
        execute.is_file(),
        "expected query execution bridge at {}",
        execute.display()
    );

    let src =
        fs::read_to_string(&execute).unwrap_or_else(|e| panic!("read {}: {e}", execute.display()));
    assert!(
        src.contains("conn.fetch_rows(&sql, &params).await"),
        "QueryBuilder::fetch_rows must delegate to driver Connection::fetch_rows"
    );
    assert!(
        src.contains("conn.fetch_optional_row(&sql, &params).await"),
        "QueryBuilder::fetch_optional_row must delegate to driver Connection::fetch_optional_row"
    );
    assert!(
        !src.contains("sqlx::"),
        "QueryBuilder execution bridge must not bypass the driver layer with sqlx"
    );
}

fn driver_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("driver")
}

fn query_execute_source() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("orm")
        .join("query")
        .join("execute.rs")
}

fn walk(dir: &Path, visit: &mut dyn FnMut(&Path)) {
    let entries = fs::read_dir(dir).unwrap_or_else(|e| panic!("read_dir {}: {e}", dir.display()));
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            walk(&path, visit);
        } else {
            visit(&path);
        }
    }
}
