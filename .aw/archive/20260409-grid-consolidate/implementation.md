---
id: implementation
type: change_implementation
change_id: grid-consolidate
---

# Implementation

## Summary

Consolidate 5 grid crates (cclab-grid-core, cclab-grid-formula, cclab-grid-history, cclab-grid-db, cclab-grid-server) into a single cclab-grid crate with sub-modules (core/, formula/, history/, db/, server/). All 466 unit tests and 16 doc tests pass. Cross-crate imports updated to intra-crate paths. Heavy dependencies feature-gated behind server and db features. cclab-grid-wasm remains separate, updated to depend on unified cclab-grid. Old crate directories removed and workspace members updated.

## Diff

```diff
diff --git a/Cargo.toml b/Cargo.toml
index 53f3e561..339721ee 100644
--- a/Cargo.toml
+++ b/Cargo.toml
@@ -11,11 +11,7 @@ members = [
     "crates/cclab-queue",
     "crates/cclab-runtime",
     "crates/cclab-schema",
-    "crates/cclab-grid-core",
-    "crates/cclab-grid-db",
-    "crates/cclab-grid-formula",
-    "crates/cclab-grid-history",
-    "crates/cclab-grid-server",
+    "crates/cclab-grid",
     "crates/cclab-grid-wasm",
     "crates/cclab-agent",
     "crates/cclab-jet",
diff --git a/crates/cclab-grid-core/Cargo.toml b/crates/cclab-grid-core/Cargo.toml
deleted file mode 100644
index 957607d1..00000000
--- a/crates/cclab-grid-core/Cargo.toml
+++ /dev/null
@@ -1,14 +0,0 @@
-[package]
-name = "cclab-grid-core"
-version.workspace = true
-edition.workspace = true
-authors.workspace = true
-license.workspace = true
-description = "Core data structures for RuSheet spreadsheet"
-
-[dependencies]
-serde.workspace = true
-serde_json.workspace = true
-thiserror.workspace = true
-bitvec = "1.0"
-regex = "1.10"
diff --git a/crates/cclab-grid-core/examples/conditional_format_example.rs b/crates/cclab-grid-core/examples/conditional_format_example.rs
deleted file mode 100644
index ea2d738b..00000000
--- a/crates/cclab-grid-core/examples/conditional_format_example.rs
+++ /dev/null
@@ -1,149 +0,0 @@
-use cclab_grid_core::{
-    Cell, CellCoord, CellFormat, CellRange, Color, ComparisonOperator, ConditionalFormat,
-    ConditionalFormattingRule, ConditionalRule, Sheet, TextOperator,
-};
-
-fn main() {
-    let mut sheet = Sheet::new("Sales Data");
-
-    // Add some sample data
-    sheet.set_cell(CellCoord::new(0, 0), Cell::text("Product"));
-    sheet.set_cell(CellCoord::new(0, 1), Cell::text("Sales"));
-    sheet.set_cell(CellCoord::new(1, 0), Cell::text("Widget A"));
-    sheet.set_cell(CellCoord::new(1, 1), Cell::number(75.0));
-    sheet.set_cell(CellCoord::new(2, 0), Cell::text("Widget B"));
-    sheet.set_cell(CellCoord::new(2, 1), Cell::number(45.0));
-    sheet.set_cell(CellCoord::new(3, 0), Cell::text("Widget C"));
-    sheet.set_cell(CellCoord::new(3, 1), Cell::number(90.0));
-    sheet.set_cell(CellCoord::new(4, 0), Cell::text("Widget D"));
-    sheet.set_cell(CellCoord::new(4, 1), Cell::number(30.0));
-
-    // Example 1: Value-based rule - highlight high sales in green
-    println!("Example 1: Value-based conditional formatting");
-    let high_sales_rule = ConditionalFormattingRule::new(
-        "high_sales".to_string(),
-        CellRange::new(CellCoord::new(1, 1), CellCoord::new(10, 1)),
-        ConditionalRule::ValueBased {
-            operator: ComparisonOperator::GreaterThanOrEqual,
-            value1: 70.0,
-            value2: None,
-            format: ConditionalFormat {
-                background_color: Some(Color::rgb(144, 238, 144)), // Light green
-                bold: Some(true),
-                ..Default::default()
-            },
-        },
-    );
-    sheet.add_conditional_formatting(high_sales_rule);
-
-    // Example 2: Value-based rule - highlight low sales in red
-    let low_sales_rule = ConditionalFormattingRule::new(
-        "low_sales".to_string(),
-        CellRange::new(CellCoord::new(1, 1), CellCoord::new(10, 1)),
-        ConditionalRule::ValueBased {
-            operator: ComparisonOperator::LessThan,
-            value1: 50.0,
-            value2: None,
-            format: ConditionalFormat {
-                background_color: Some(Color::rgb(255, 182, 193)), // Light red
-                italic: Some(true),
-                ..Default::default()
-            },
-        },
-    );
-    sheet.add_conditional_formatting(low_sales_rule);
-
-    // Example 3: Text-based rule - highlight product names containing "A"
-    println!("\nExample 2: Text-based conditional formatting");
-    let text_rule = ConditionalFormattingRule::new(
-        "contains_a".to_string(),
-        CellRange::new(CellCoord::new(1, 0), CellCoord::new(10, 0)),
-        ConditionalRule::TextBased {
-            operator: TextOperator::Contains,
-            pattern: Some("A".to_string()),
-            case_sensitive: false,
-            format: ConditionalFormat {
-                text_color: Some(Color::rgb(0, 0, 255)), // Blue
-                underline: Some(true),
-                ..Default::default()
-            },
-        },
-    );
-    sheet.add_conditional_formatting(text_rule);
-
-    // Example 4: Color scale - create a gradient based on sales values
-    println!("\nExample 3: Color scale conditional formatting");
-
-    // For demonstration, let's create a separate sheet for color scale
-    let mut color_sheet = Sheet::new("Color Scale Example");
-    color_sheet.set_cell(CellCoord::new(0, 0), Cell::number(0.0));
-    color_sheet.set_cell(CellCoord::new(0, 1), Cell::number(25.0));
-    color_sheet.set_cell(CellCoord::new(0, 2), Cell::number(50.0));
-    color_sheet.set_cell(CellCoord::new(0, 3), Cell::number(75.0));
-    color_sheet.set_cell(CellCoord::new(0, 4), Cell::number(100.0));
-
-    let color_scale_rule = ConditionalFormattingRule::new(
-        "sales_gradient".to_string(),
-        CellRange::new(CellCoord::new(0, 0), CellCoord::new(0, 4)),
-        ConditionalRule::ColorScale {
-            min_color: Color::rgb(255, 0, 0),     // Red for low values
-            max_color: Color::rgb(0, 255, 0),     // Green for high values
-            mid_color: Some(Color::rgb(255, 255, 0)), // Yellow for mid values
-        },
-    );
-    color_sheet.add_conditional_formatting(color_scale_rule);
-
-    // Test effective format calculation
-    println!("\nTesting effective format calculation:");
-    let base_format = CellFormat::default();
-
-    // Widget A (sales: 75) - should be green and bold (high sales)
-    let value = sheet.get_cell_value(CellCoord::new(1, 1));
-    let effective = sheet.get_effective_format(1, 1, &base_format, value);
-    println!("Widget A (75): Bold={}, Background={:?}",
-        effective.bold,
-        effective.background_color.map(|c| c.to_hex())
-    );
-
-    // Widget D (sales: 30) - should be light red and italic (low sales)
-    let value = sheet.get_cell_value(CellCoord::new(4, 1));
-    let effective = sheet.get_effective_format(4, 1, &base_format, value);
-    println!("Widget D (30): Italic={}, Background={:?}",
-        effective.italic,
-        effective.background_color.map(|c| c.to_hex())
-    );
-
-    // Widget A name - should be blue and underlined (contains "A")
-    let value = sheet.get_cell_value(CellCoord::new(1, 0));
-    let effective = sheet.get_effective_format(1, 0, &base_format, value);
-    println!("Widget A name: Underline={}, TextColor={:?}",
-        effective.underline,
-        effective.text_color.map(|c| c.to_hex())
-    );
-
-    // Test color scale
-    println!("\nColor scale values:");
-    for col in 0..5 {
-        let value = color_sheet.get_cell_value(CellCoord::new(0, col));
-        let effective = color_sheet.get_effective_format(0, col, &base_format, value);
-        if let Some(color) = effective.background_color {
-            println!("Value {}: Color={}", value.as_number().unwrap(), color.to_hex());
-        }
-    }
-
-    // Show all conditional formatting rules
-    println!("\nAll conditional formatting rules on sheet:");
-    for rule in sheet.get_conditional_formatting_rules() {
-        println!("  - Rule ID: {}, Range: {}, Enabled: {}, Priority: {}",
-            rule.id,
-            rule.range.to_a1(),
-            rule.enabled,
-            rule.priority
-        );
-    }
-
-    // Example of removing a rule
-    println!("\nRemoving 'low_sales' rule...");
-    sheet.remove_conditional_formatting("low_sales");
-    println!("Remaining rules: {}", sheet.get_conditional_formatting_rules().len());
-}
diff --git a/crates/cclab-grid-core/examples/search_demo.rs b/crates/cclab-grid-core/examples/search_demo.rs
deleted file mode 100644
index 8dbfa42a..00000000
--- a/crates/cclab-grid-core/examples/search_demo.rs
+++ /dev/null
@@ -1,117 +0,0 @@
-use cclab_grid_core::{Cell, CellCoord, SearchEngine, SearchOptions, ReplaceOptions, Workbook};
-
-fn main() {
-    // Create a workbook with sample data
-    let mut workbook = Workbook::new("SearchDemo");
-
-    let sheet = workbook.active_sheet_mut();
-    sheet.set_cell(CellCoord::new(0, 0), Cell::text("Apple"));
-    sheet.set_cell(CellCoord::new(0, 1), Cell::text("Banana"));
-    sheet.set_cell(CellCoord::new(0, 2), Cell::text("Cherry"));
-    sheet.set_cell(CellCoord::new(1, 0), Cell::text("apple pie"));
-    sheet.set_cell(CellCoord::new(1, 1), Cell::number(42.0));
-    sheet.set_cell(CellCoord::new(1, 2), Cell::text("Price: $42"));
-    sheet.set_cell(CellCoord::new(2, 0), Cell::formula("=SUM(B2:B3)"));
-
-    // Example 1: Simple case-insensitive search
-    println!("=== Example 1: Case-insensitive search for 'apple' ===");
-    let options = SearchOptions {
-        query: "apple".to_string(),
-        match_case: false,
-        match_entire_cell: false,
-        use_regex: false,
-        search_formulas: false,
-        sheet_indices: None,
-    };
-
-    let results = SearchEngine::search(&workbook, &options).unwrap();
-    println!("Found {} matches:", results.len());
-    for result in &results {
-        println!("  - Sheet: {}, Cell: {}:{} -> '{}'",
-            result.sheet_name, result.row, result.col, result.cell_value);
-    }
-
-    // Example 2: Regex search for numbers
-    println!("\n=== Example 2: Regex search for numbers ===");
-    let options = SearchOptions {
-        query: r"\d+".to_string(),
-        match_case: false,
-        match_entire_cell: false,
-        use_regex: true,
-        search_formulas: false,
-        sheet_indices: None,
-    };
-
-    let results = SearchEngine::search(&workbook, &options).unwrap();
-    println!("Found {} matches:", results.len());
-    for result in &results {
-        println!("  - Sheet: {}, Cell: {}:{} -> '{}' (matched: '{}')",
-            result.sheet_name, result.row, result.col, result.cell_value, result.matched_text);
-    }
-
-    // Example 3: Search formulas
-    println!("\n=== Example 3: Search in formulas ===");
-    let options = SearchOptions {
-        query: "SUM".to_string(),
-        match_case: false,
-        match_entire_cell: false,
-        use_regex: false,
-        search_formulas: true,
-        sheet_indices: None,
-    };
-
-    let results = SearchEngine::search(&workbook, &options).unwrap();
-    println!("Found {} matches:", results.len());
-    for result in &results {
-        println!("  - Sheet: {}, Cell: {}:{} -> '{}' (is_formula: {})",
-            result.sheet_name, result.row, result.col, result.cell_value, result.is_formula);
-    }
-
-    // Example 4: Replace text
-    println!("\n=== Example 4: Replace 'apple' with 'orange' ===");
-    let options = ReplaceOptions {
-        search: SearchOptions {
-            query: "apple".to_string(),
-            match_case: false,
-            match_entire_cell: false,
-            use_regex: false,
-            search_formulas: false,
-            sheet_indices: None,
-        },
-        replacement: "orange".to_string(),
-    };
-
-    let results = SearchEngine::replace(&mut workbook, &options).unwrap();
-    println!("Replaced {} cells:", results.len());
-    for result in &results {
-        println!("  - Sheet: {}, Cell: {}:{} -> '{}'",
-            result.sheet_name, result.row, result.col, result.cell_value);
-    }
-
-    // Verify the replacements
-    println!("\n=== Verifying replacements ===");
-    let sheet = workbook.active_sheet();
-    println!("A1: {}", sheet.get_cell(CellCoord::new(0, 0)).unwrap().content.display_value());
-    println!("A2: {}", sheet.get_cell(CellCoord::new(1, 0)).unwrap().content.display_value());
-
-    // Example 5: Regex replace
-    println!("\n=== Example 5: Replace numbers with 'XX' using regex ===");
-    let options = ReplaceOptions {
-        search: SearchOptions {
-            query: r"\d+".to_string(),
-            match_case: false,
-            match_entire_cell: false,
-            use_regex: true,
-            search_formulas: false,
-            sheet_indices: None,
-        },
-        replacement: "XX".to_string(),
-    };
-
-    let results = SearchEngine::replace(&mut workbook, &options).unwrap();
-    println!("Replaced {} cells:", results.len());
-    for result in &results {
-        println!("  - Sheet: {}, Cell: {}:{} -> '{}'",
-            result.sheet_name, result.row, result.col, result.cell_value);
-    }
-}
diff --git a/crates/cclab-grid-db/Cargo.toml b/crates/cclab-grid-db/Cargo.toml
deleted file mode 100644
index d005b7d7..00000000
--- a/crates/cclab-grid-db/Cargo.toml
+++ /dev/null
@@ -1,19 +0,0 @@
-[package]
-name = "cclab-grid-db"
-version = "0.1.0"
-edition = "2021"
-description = "Spreadsheet database layer with Morton encoding and WAL-backed persistence"
-
-[dependencies]
-cclab-wal = { path = "../cclab-wal" }
-cclab-grid-core = { path = "../cclab-grid-core" }
-serde = { version = "1.0", features = ["derive"] }
-serde_json = "1.0"
-thiserror = "2.0"
-tokio = { version = "1.40", features = ["full"] }
-bincode = "1.3"
-tracing = "0.1"
-parking_lot = "0.12"
-
-[dev-dependencies]
-tempfile = "3.8"
diff --git a/crates/cclab-grid-formula/Cargo.toml b/crates/cclab-grid-formula/Cargo.toml
deleted file mode 100644
index 71ee70c3..00000000
--- a/crates/cclab-grid-formula/Cargo.toml
+++ /dev/null
@@ -1,13 +0,0 @@
-[package]
-name = "cclab-grid-formula"
-version.workspace = true
-edition.workspace = true
-authors.workspace = true
-license.workspace = true
-description = "Formula parsing and evaluation engine for RuSheet"
-
-[dependencies]
-cclab-grid-core = { path = "../cclab-grid-core" }
-serde.workspace = true
-thiserror.workspace = true
-nom = "7.1"
diff --git a/crates/cclab-grid-history/Cargo.toml b/crates/cclab-grid-history/Cargo.toml
deleted file mode 100644
index 682e41ff..00000000
--- a/crates/cclab-grid-history/Cargo.toml
+++ /dev/null
@@ -1,11 +0,0 @@
-[package]
-name = "cclab-grid-history"
-version.workspace = true
-edition.workspace = true
-authors.workspace = true
-license.workspace = true
-description = "Undo/redo history management for RuSheet"
-
-[dependencies]
-cclab-grid-core = { path = "../cclab-grid-core" }
-cclab-grid-formula = { path = "../cclab-grid-formula" }
diff --git a/crates/cclab-grid-server/.env.example b/crates/cclab-grid-server/.env.example
deleted file mode 100644
index 7cc6c25e..00000000
--- a/crates/cclab-grid-server/.env.example
+++ /dev/null
@@ -1,9 +0,0 @@
-# Server configuration
-HOST=0.0.0.0
-PORT=3000
-
-# Database connection
-DATABASE_URL=postgres://postgres:postgres@localhost:5432/rusheet
-
-# Logging level
-RUST_LOG=rusheet_server=debug,tower_http=debug
diff --git a/crates/cclab-grid-server/Cargo.toml b/crates/cclab-grid-server/Cargo.toml
deleted file mode 100644
index d557127f..00000000
--- a/crates/cclab-grid-server/Cargo.toml
+++ /dev/null
@@ -1,40 +0,0 @@
-[package]
-name = "cclab-grid-server"
-version.workspace = true
-edition.workspace = true
-authors.workspace = true
-license.workspace = true
-description = "RuSheet collaboration and data server"
-
-[dependencies]
-# Workspace crates
-cclab-grid-core = { path = "../cclab-grid-core" }
-cclab-grid-db = { path = "../cclab-grid-db" }
-cclab-kv = { path = "../cclab-kv" }
-
-# Web framework
-axum = { version = "0.7", features = ["ws", "macros"] }
-tokio = { version = "1", features = ["full"] }
-tower = "0.5"
-tower-http = { version = "0.6", features = ["cors", "trace"] }
-
-# CRDT
-yrs = "0.21"
-
-# Serialization
-serde = { workspace = true }
-serde_json = { workspace = true }
-
-# Utilities
-uuid = { version = "1", features = ["v4", "serde"] }
-chrono = { version = "0.4", features = ["serde"] }
-tracing = "0.1"
-tracing-subscriber = { version = "0.3", features = ["env-filter"] }
-dotenvy = "0.15"
-thiserror = { workspace = true }
-anyhow = "1.0"
-futures = "0.3"
-
-[[bin]]
-name = "cclab-grid-server"
-path = "src/main.rs"
diff --git a/crates/cclab-grid-wasm/Cargo.toml b/crates/cclab-grid-wasm/Cargo.toml
index 46418a4e..baaa3cf2 100644
--- a/crates/cclab-grid-wasm/Cargo.toml
+++ b/crates/cclab-grid-wasm/Cargo.toml
@@ -10,9 +10,7 @@ description = "WebAssembly bindings for RuSheet"
 crate-type = ["cdylib", "rlib"]
 
 [dependencies]
-cclab-grid-core = { path = "../cclab-grid-core" }
-cclab-grid-formula = { path = "../cclab-grid-formula" }
-cclab-grid-history = { path = "../cclab-grid-history" }
+cclab-grid = { path = "../cclab-grid", default-features = false }
 wasm-bindgen = "0.2"
 js-sys = "0.3"
 serde.workspace = true
diff --git a/crates/cclab-grid-wasm/src/api.rs b/crates/cclab-grid-wasm/src/api.rs
index c90c5dbd..89f4a0f0 100644
--- a/crates/cclab-grid-wasm/src/api.rs
+++ b/crates/cclab-grid-wasm/src/api.rs
@@ -1,12 +1,12 @@
-use cclab_grid_core::{
+use cclab_grid::core::{
     CellContent, CellCoord, CellError, CellFormat, CellRange, CellValue,
     CellPosition, Selection,
     ConditionalFormattingRule, ConditionalRule, HorizontalAlign, RusheetError,
     VerticalAlign, Workbook, DataValidationRule, ValidationCriteria, ValidationResult,
     ValidationAlert, ValidationMessage, AlertStyle,
 };
-use cclab_grid_formula::{extract_references, DependencyGraph};
-use cclab_grid_history::{
+use cclab_grid::formula::{extract_references, DependencyGraph};
+use cclab_grid::history::{
     ApplyFilterCommand, ClearFilterCommand, ClearRangeCommand, DeleteColsCommand,
     DeleteRowsCommand, HistoryManager, InsertColsCommand, InsertRowsCommand, MergeCellsCommand,
     SetCellFormatCommand, SetCellValueCommand, SetRangeFormatCommand, SortRangeCommand,
@@ -127,7 +127,7 @@ pub struct MergeInfo {
 
 impl From<&CellFormat> for CellFormatData {
     fn from(format: &CellFormat) -> Self {
-        use cclab_grid_core::{HorizontalAlign, VerticalAlign};
+        use cclab_grid::core::{HorizontalAlign, VerticalAlign};
 
         CellFormatData {
             bold: format.bold,
@@ -259,7 +259,7 @@ impl SpreadsheetEngine {
             ).into());
 
             // Create closure to get cell values from any sheet
-            let result = cclab_grid_formula::evaluate_formula_cross_sheet(
+            let result = cclab_grid::formula::evaluate_formula_cross_sheet(
                 &expression,
                 Some(&current_sheet_name),
                 |sheet_name, r, c| {
@@ -899,10 +899,10 @@ impl SpreadsheetEngine {
     /// Returns JSON array of SearchResult
     #[wasm_bindgen]
     pub fn search(&self, options_json: &str) -> Result<String, JsValue> {
-        let options: cclab_grid_core::SearchOptions = serde_json::from_str(options_json)
+        let options: cclab_grid::core::SearchOptions = serde_json::from_str(options_json)
             .map_err(JsRuSheetError::from_error)?;
 
-        let results = cclab_grid_core::SearchEngine::search(&self.workbook, &options)
+        let results = cclab_grid::core::SearchEngine::search(&self.workbook, &options)
             .map_err(JsRuSheetError::from_error)?;
 
         serde_json::to_string(&results)
@@ -925,10 +925,10 @@ impl SpreadsheetEngine {
     /// Returns JSON array of replaced cells (SearchResult format)
     #[wasm_bindgen]
     pub fn replace(&mut self, options_json: &str) -> Result<String, JsValue> {
-        let options: cclab_grid_core::ReplaceOptions = serde_json::from_str(options_json)
+        let options: cclab_grid::core::ReplaceOptions = serde_json::from_str(options_json)
             .map_err(JsRuSheetError::from_error)?;
 
-        let results = cclab_grid_core::SearchEngine::replace(&mut self.workbook, &options)
+        let results = cclab_grid::core::SearchEngine::replace(&mut self.workbook, &options)
             .map_err(JsRuSheetError::from_error)?;
 
         serde_json::to_string(&results)
@@ -938,7 +938,7 @@ impl SpreadsheetEngine {
     /// Simple search in current sheet only
     #[wasm_bindgen(js_name = searchCurrentSheet)]
     pub fn search_current_sheet(&self, query: &str, match_case: bool) -> Result<String, JsValue> {
-        let options = cclab_grid_core::SearchOptions {
+        let options = cclab_grid::core::SearchOptions {
             query: query.to_string(),
             match_case,
             match_entire_cell: false,
@@ -947,7 +947,7 @@ impl SpreadsheetEngine {
             sheet_indices: Some(vec![self.workbook.active_sheet_index]),
         };
 
-        let results = cclab_grid_core::SearchEngine::search(&self.workbook, &options)
+        let results = cclab_grid::core::SearchEngine::search(&self.workbook, &options)
             .map_err(JsRuSheetError::from_error)?;
 
         serde_json::to_string(&results)
@@ -1526,7 +1526,7 @@ impl SpreadsheetEngine {
 
 /// Convert CellFormatData to CellFormat
 fn cell_format_from_data(data: &CellFormatData) -> CellFormat {
-    use cclab_grid_core::{Color, HorizontalAlign, VerticalAlign};
+    use cclab_grid::core::{Color, HorizontalAlign, VerticalAlign};
 
     let mut format = CellFormat::default();
     format.bold = data.bold;
@@ -1631,7 +1631,7 @@ mod tests {
 
 #[cfg(test)]
 mod bug_fixes {
-    use cclab_grid_core::{CellCoord, Sheet};
+    use cclab_grid::core::{CellCoord, Sheet};
 
     #[test]
     fn test_bug_1_3_number_preservation() {
@@ -1640,8 +1640,8 @@ mod bug_fixes {
 
         // Enter "10" in B2 (row=1, col=1)
         let coord = CellCoord::new(1, 1);
-        let content = cclab_grid_core::sheet::parse_cell_input("10");
-        let cell = cclab_grid_core::Cell::new(content);
+        let content = cclab_grid::core::sheet::parse_cell_input("10");
+        let cell = cclab_grid::core::Cell::new(content);
         sheet.set_cell(coord, cell);
 
         // Verify original_input returns "10"
@@ -1653,8 +1653,8 @@ mod bug_fixes {
 
         // Enter formula "=B2" in B3
         let coord_b3 = CellCoord::new(1, 2);
-        let formula_content = cclab_grid_core::sheet::parse_cell_input("=B2");
-        let formula_cell = cclab_grid_core::Cell::new(formula_content);
+        let formula_content = cclab_grid::core::sheet::parse_cell_input("=B2");
+        let formula_cell = cclab_grid::core::Cell::new(formula_content);
         sheet.set_cell(coord_b3, formula_cell);
 
         // Re-check B2 - should STILL be "10", not corrupted
@@ -1670,33 +1670,33 @@ mod bug_fixes {
         let mut sheet = Sheet::new("Test");
 
         // Test number
-        let content = cclab_grid_core::sheet::parse_cell_input("42");
-        sheet.set_cell(CellCoord::new(0, 0), cclab_grid_core::Cell::new(content));
+        let content = cclab_grid::core::sheet::parse_cell_input("42");
+        sheet.set_cell(CellCoord::new(0, 0), cclab_grid::core::Cell::new(content));
         let cell = sheet.get_cell(CellCoord::new(0, 0)).unwrap();
         assert_eq!(cell.content.original_input(), "42");
 
         // Test percentage
-        let content = cclab_grid_core::sheet::parse_cell_input("50%");
-        sheet.set_cell(CellCoord::new(0, 1), cclab_grid_core::Cell::new(content));
+        let content = cclab_grid::core::sheet::parse_cell_input("50%");
+        sheet.set_cell(CellCoord::new(0, 1), cclab_grid::core::Cell::new(content));
         let cell = sheet.get_cell(CellCoord::new(0, 1)).unwrap();
         assert_eq!(cell.content.original_input(), "50%");
         assert_eq!(cell.content.display_value(), "0.5");
 
         // Test boolean
-        let content = cclab_grid_core::sheet::parse_cell_input("TRUE");
-        sheet.set_cell(CellCoord::new(0, 2), cclab_grid_core::Cell::new(content));
+        let content = cclab_grid::core::sheet::parse_cell_input("TRUE");
+        sheet.set_cell(CellCoord::new(0, 2), cclab_grid::core::Cell::new(content));
         let cell = sheet.get_cell(CellCoord::new(0, 2)).unwrap();
         assert_eq!(cell.content.original_input(), "TRUE");
 
         // Test text
-        let content = cclab_grid_core::sheet::parse_cell_input("Hello");
-        sheet.set_cell(CellCoord::new(0, 3), cclab_grid_core::Cell::new(content));
+        let content = cclab_grid::core::sheet::parse_cell_input("Hello");
+        sheet.set_cell(CellCoord::new(0, 3), cclab_grid::core::Cell::new(content));
         let cell = sheet.get_cell(CellCoord::new(0, 3)).unwrap();
         assert_eq!(cell.content.original_input(), "Hello");
 
         // Test formula
-        let content = cclab_grid_core::sheet::parse_cell_input("=A1+B1");
-        sheet.set_cell(CellCoord::new(0, 4), cclab_grid_core::Cell::new(content));
+        let content = cclab_grid::core::sheet::parse_cell_input("=A1+B1");
+        sheet.set_cell(CellCoord::new(0, 4), cclab_grid::core::Cell::new(content));
         let cell = sheet.get_cell(CellCoord::new(0, 4)).unwrap();
         assert_eq!(cell.content.original_input(), "=A1+B1");
     }
@@ -1865,7 +1865,7 @@ mod bug_fixes {
 
     #[test]
     fn test_data_validation_integration() {
-        use cclab_grid_core::{DataValidationRule, ValidationCriteria, ValidationAlert, AlertStyle, ListSource, CellRange, CellCoord};
+        use cclab_grid::core::{DataValidationRule, ValidationCriteria, ValidationAlert, AlertStyle, ListSource, CellRange, CellCoord};
 
         let mut engine = super::SpreadsheetEngine::new();
 
@@ -1929,7 +1929,7 @@ mod bug_fixes {
 
     #[test]
     fn test_data_validation_number_range() {
-        use cclab_grid_core::{DataValidationRule, ValidationCriteria, ValidationAlert, AlertStyle, ValidationOperator, CellRange, CellCoord};
+        use cclab_grid::core::{DataValidationRule, ValidationCriteria, ValidationAlert, AlertStyle, ValidationOperator, CellRange, CellCoord};
 
         let mut engine = super::SpreadsheetEngine::new();
 
@@ -1983,7 +1983,7 @@ mod bug_fixes {
 
     #[test]
     fn test_conditional_formatting_integration() {
-        use cclab_grid_core::{CellRange, Color, ComparisonOperator, ConditionalFormat, ConditionalFormattingRule, ConditionalRule};
+        use cclab_grid::core::{CellRange, Color, ComparisonOperator, ConditionalFormat, ConditionalFormattingRule, ConditionalRule};
 
         let mut engine = super::SpreadsheetEngine::new();
 
diff --git a/crates/cclab-grid/Cargo.toml b/crates/cclab-grid/Cargo.toml
new file mode 100644
index 00000000..cc09c594
--- /dev/null
+++ b/crates/cclab-grid/Cargo.toml
@@ -0,0 +1,53 @@
+[package]
+name = "cclab-grid"
+version.workspace = true
+edition.workspace = true
+authors.workspace = true
+license.workspace = true
+description = "Spreadsheet engine: core data structures, formula evaluation, history, persistence, and collaboration server"
+
+[features]
+default = ["db", "server"]
+db = ["dep:cclab-wal", "dep:bincode", "dep:parking_lot"]
+server = ["db", "dep:cclab-kv", "dep:axum", "dep:tower", "dep:tower-http", "dep:yrs", "dep:uuid", "dep:chrono", "dep:tracing-subscriber", "dep:dotenvy", "dep:anyhow", "dep:futures"]
+
+[dependencies]
+# Core dependencies
+serde.workspace = true
+serde_json.workspace = true
+thiserror.workspace = true
+bitvec = "1.0"
+regex = "1.10"
+
+# Formula dependencies
+nom = "7.1"
+
+# Shared dependencies
+tokio = { version = "1.40", features = ["full"] }
+tracing = "0.1"
+
+# DB dependencies (feature-gated)
+cclab-wal = { path = "../cclab-wal", optional = true }
+bincode = { version = "1.3", optional = true }
+parking_lot = { version = "0.12", optional = true }
+
+# Server dependencies (feature-gated)
+cclab-kv = { path = "../cclab-kv", optional = true }
+axum = { version = "0.7", features = ["ws", "macros"], optional = true }
+tower = { version = "0.5", optional = true }
+tower-http = { version = "0.6", features = ["cors", "trace"], optional = true }
+yrs = { version = "0.21", optional = true }
+uuid = { version = "1", features = ["v4", "serde"], optional = true }
+chrono = { version = "0.4", features = ["serde"], optional = true }
+tracing-subscriber = { version = "0.3", features = ["env-filter"], optional = true }
+dotenvy = { version = "0.15", optional = true }
+anyhow = { version = "1.0", optional = true }
+futures = { version = "0.3", optional = true }
+
+[dev-dependencies]
+tempfile = "3.8"
+
+[[bin]]
+name = "cclab-grid-server"
+path = "src/bin/cclab-grid-server.rs"
+required-features = ["server"]
diff --git a/crates/cclab-grid-server/src/main.rs b/crates/cclab-grid/src/bin/cclab-grid-server.rs
similarity index 92%
rename from crates/cclab-grid-server/src/main.rs
rename to crates/cclab-grid/src/bin/cclab-grid-server.rs
index 9896b848..4a1c0402 100644
--- a/crates/cclab-grid-server/src/main.rs
+++ b/crates/cclab-grid/src/bin/cclab-grid-server.rs
@@ -1,4 +1,4 @@
-use cclab_grid_server::{config::Config, run_server};
+use cclab_grid::server::{config::Config, run_server};
 use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
 
 #[tokio::main]
diff --git a/crates/cclab-grid-core/src/cell.rs b/crates/cclab-grid/src/core/cell.rs
similarity index 98%
rename from crates/cclab-grid-core/src/cell.rs
rename to crates/cclab-grid/src/core/cell.rs
index bc3eb56e..ccef1cf5 100644
--- a/crates/cclab-grid-core/src/cell.rs
+++ b/crates/cclab-grid/src/core/cell.rs
@@ -1,7 +1,7 @@
 use serde::{Deserialize, Serialize};
 
-use crate::error::CellError;
-use crate::format::CellFormat;
+use crate::core::error::CellError;
+use crate::core::format::CellFormat;
 
 /// Represents the raw value stored in a cell
 #[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
diff --git a/crates/cclab-grid-core/src/chunk.rs b/crates/cclab-grid/src/core/chunk.rs
similarity index 99%
rename from crates/cclab-grid-core/src/chunk.rs
rename to crates/cclab-grid/src/core/chunk.rs
index c98f6869..b10b050a 100644
--- a/crates/cclab-grid-core/src/chunk.rs
+++ b/crates/cclab-grid/src/core/chunk.rs
@@ -11,7 +11,7 @@ use std::collections::HashMap;
 
 use bitvec::prelude::*;
 
-use crate::spatial::morton_encode;
+use crate::core::spatial::morton_encode;
 
 /// Size of each chunk in both dimensions (64x64 cells per chunk).
 pub const CHUNK_SIZE: usize = 64;
@@ -44,7 +44,7 @@ impl ChunkCoord {
     /// # Examples
     ///
     /// ```
-    /// use cclab_grid_core::ChunkCoord;
+    /// use cclab_grid::core::ChunkCoord;
     ///
     /// let coord = ChunkCoord::from_cell(64, 128);
     /// assert_eq!(coord.block_row, 1);
@@ -176,7 +176,7 @@ impl<T> Chunk<T> {
         self.occupied
             .iter_ones()
             .filter_map(move |idx| {
-                let (row, col) = crate::spatial::morton_decode(idx as u16);
+                let (row, col) = crate::core::spatial::morton_decode(idx as u16);
                 self.cells[idx].as_ref().map(|v| ((row, col), v))
             })
     }
@@ -189,7 +189,7 @@ impl<T> Chunk<T> {
         self.occupied
             .iter_ones()
             .filter_map(move |idx| {
-                let (row, col) = crate::spatial::morton_decode(idx as u16);
+                let (row, col) = crate::core::spatial::morton_decode(idx as u16);
                 // SAFETY: We only iterate over occupied indices
                 let cell_ptr = cells.as_mut_ptr();
                 unsafe { (*cell_ptr.add(idx)).as_mut().map(|v| ((row, col), v)) }
diff --git a/crates/cclab-grid-core/src/conditional_format.rs b/crates/cclab-grid/src/core/conditional_format.rs
similarity index 99%
rename from crates/cclab-grid-core/src/conditional_format.rs
rename to crates/cclab-grid/src/core/conditional_format.rs
index 9b240f15..d6e240e8 100644
--- a/crates/cclab-grid-core/src/conditional_format.rs
+++ b/crates/cclab-grid/src/core/conditional_format.rs
@@ -1,4 +1,4 @@
-use crate::{CellValue, CellFormat, Color, CellRange};
+use crate::core::{CellValue, CellFormat, Color, CellRange};
 use serde::{Deserialize, Serialize};
 
 /// Comparison operators for value-based rules
@@ -268,7 +268,7 @@ impl ConditionalFormattingRule {
 #[cfg(test)]
 mod tests {
     use super::*;
-    use crate::CellCoord;
+    use crate::core::CellCoord;
 
     #[test]
     fn test_value_based_greater_than() {
diff --git a/crates/cclab-grid-core/src/error.rs b/crates/cclab-grid/src/core/error.rs
similarity index 100%
rename from crates/cclab-grid-core/src/error.rs
rename to crates/cclab-grid/src/core/error.rs
diff --git a/crates/cclab-grid-core/src/format.rs b/crates/cclab-grid/src/core/format.rs
similarity index 100%
rename from crates/cclab-grid-core/src/format.rs
rename to crates/cclab-grid/src/core/format.rs
diff --git a/crates/cclab-grid-core/src/gap_buffer.rs b/crates/cclab-grid/src/core/gap_buffer.rs
similarity index 98%
rename from crates/cclab-grid-core/src/gap_buffer.rs
rename to crates/cclab-grid/src/core/gap_buffer.rs
index af1216d0..423addf5 100644
--- a/crates/cclab-grid-core/src/gap_buffer.rs
+++ b/crates/cclab-grid/src/core/gap_buffer.rs
@@ -7,7 +7,7 @@
 //! # Example
 //!
 //! ```
-//! use cclab_grid_core::GapBuffer;
+//! use cclab_grid::core::GapBuffer;
 //!
 //! let mut buffer = GapBuffer::with_size(5);
 //! assert_eq!(buffer.logical_to_physical(2), Some(2)); // Initially 1:1 mapping
@@ -59,7 +59,7 @@ impl GapBuffer {
     /// # Example
     ///
     /// ```
-    /// use cclab_grid_core::GapBuffer;
+    /// use cclab_grid::core::GapBuffer;
     ///
     /// let buffer = GapBuffer::with_size(5);
     /// assert_eq!(buffer.len(), 5);
@@ -89,7 +89,7 @@ impl GapBuffer {
     /// # Example
     ///
     /// ```
-    /// use cclab_grid_core::GapBuffer;
+    /// use cclab_grid::core::GapBuffer;
     ///
     /// let mut buffer = GapBuffer::with_size(3);
     /// buffer.insert_at(1, 2); // Insert 2 elements at position 1
@@ -150,7 +150,7 @@ impl GapBuffer {
     /// # Example
     ///
     /// ```
-    /// use cclab_grid_core::GapBuffer;
+    /// use cclab_grid::core::GapBuffer;
     ///
     /// let mut buffer = GapBuffer::with_size(5);
     /// buffer.delete_at(1, 2); // Delete 2 elements starting at position 1
@@ -211,7 +211,7 @@ impl GapBuffer {
     /// # Example
     ///
     /// ```
-    /// use cclab_grid_core::GapBuffer;
+    /// use cclab_grid::core::GapBuffer;
     ///
     /// let buffer = GapBuffer::with_size(3);
     /// assert_eq!(buffer.logical_to_physical(0), Some(0));
@@ -229,7 +229,7 @@ impl GapBuffer {
     /// # Example
     ///
     /// ```
-    /// use cclab_grid_core::GapBuffer;
+    /// use cclab_grid::core::GapBuffer;
     ///
     /// let buffer = GapBuffer::with_size(3);
     /// assert_eq!(buffer.physical_to_logical(0), Some(0));
diff --git a/crates/cclab-grid-core/src/lib.rs b/crates/cclab-grid/src/core/mod.rs
similarity index 100%
rename from crates/cclab-grid-core/src/lib.rs
rename to crates/cclab-grid/src/core/mod.rs
diff --git a/crates/cclab-grid-core/src/range.rs b/crates/cclab-grid/src/core/range.rs
similarity index 100%
rename from crates/cclab-grid-core/src/range.rs
rename to crates/cclab-grid/src/core/range.rs
diff --git a/crates/cclab-grid-core/src/search.rs b/crates/cclab-grid/src/core/search.rs
similarity index 99%
rename from crates/cclab-grid-core/src/search.rs
rename to crates/cclab-grid/src/core/search.rs
index 96411533..9f8154db 100644
--- a/crates/cclab-grid-core/src/search.rs
+++ b/crates/cclab-grid/src/core/search.rs
@@ -1,4 +1,4 @@
-use crate::{CellContent, Sheet, Workbook};
+use crate::core::{CellContent, Sheet, Workbook};
 use regex::Regex;
 use serde::{Deserialize, Serialize};
 use std::fmt;
@@ -398,7 +398,7 @@ impl SearchEngine {
 #[cfg(test)]
 mod tests {
     use super::*;
-    use crate::{Cell, CellCoord};
+    use crate::core::{Cell, CellCoord};
 
     fn create_test_workbook() -> Workbook {
         let mut workbook = Workbook::new("Test");
diff --git a/crates/cclab-grid-core/src/sheet.rs b/crates/cclab-grid/src/core/sheet.rs
similarity index 98%
rename from crates/cclab-grid-core/src/sheet.rs
rename to crates/cclab-grid/src/core/sheet.rs
index 496d68c0..40ec1509 100644
--- a/crates/cclab-grid-core/src/sheet.rs
+++ b/crates/cclab-grid/src/core/sheet.rs
@@ -1,13 +1,13 @@
 use serde::{Deserialize, Serialize};
 use std::collections::{HashMap, HashSet};
 
-use crate::cell::{Cell, CellContent, CellValue};
-use crate::chunk::ChunkedGrid;
-use crate::conditional_format::ConditionalFormattingRule;
-use crate::format::CellFormat;
-use crate::range::{CellCoord, CellRange};
-use crate::spatial::SpatialIndex;
-use crate::validation::{DataValidationRule, ValidationResult};
+use crate::core::cell::{Cell, CellContent, CellValue};
+use crate::core::chunk::ChunkedGrid;
+use crate::core::conditional_format::ConditionalFormattingRule;
+use crate::core::format::CellFormat;
+use crate::core::range::{CellCoord, CellRange};
+use crate::core::spatial::SpatialIndex;
+use crate::core::validation::{DataValidationRule, ValidationResult};
 
 /// Represents a filter applied to a column
 #[derive(Debug, Clone, Serialize, Deserialize)]
@@ -1750,7 +1750,7 @@ mod tests {
 
     #[test]
     fn test_conditional_formatting_add_remove() {
-        use crate::{ConditionalFormattingRule, ConditionalRule, ConditionalFormat, ComparisonOperator, Color};
+        use crate::core::{ConditionalFormattingRule, ConditionalRule, ConditionalFormat, ComparisonOperator, Color};
 
         let mut sheet = Sheet::new("Test");
 
@@ -1782,7 +1782,7 @@ mod tests {
 
     #[test]
     fn test_conditional_formatting_effective_format() {
-        use crate::{ConditionalFormattingRule, ConditionalRule, ConditionalFormat, ComparisonOperator, Color};
+        use crate::core::{ConditionalFormattingRule, ConditionalRule, ConditionalFormat, ComparisonOperator, Color};
 
         let mut sheet = Sheet::new("Test");
 
@@ -1827,7 +1827,7 @@ mod tests {
 
     #[test]
     fn test_conditional_formatting_priority() {
-        use crate::{ConditionalFormattingRule, ConditionalRule, ConditionalFormat, ComparisonOperator, Color};
+        use crate::core::{ConditionalFormattingRule, ConditionalRule, ConditionalFormat, ComparisonOperator, Color};
 
         let mut sheet = Sheet::new("Test");
         sheet.set_cell(CellCoord::new(0, 0), Cell::number(60.0));
@@ -1877,7 +1877,7 @@ mod tests {
 
     #[test]
     fn test_conditional_formatting_color_scale() {
-        use crate::{ConditionalFormattingRule, ConditionalRule, Color};
+        use crate::core::{ConditionalFormattingRule, ConditionalRule, Color};
 
         let mut sheet = Sheet::new("Test");
 
@@ -1924,7 +1924,7 @@ mod tests {
 
     #[test]
     fn test_data_validation_add_remove() {
-        use crate::{DataValidationRule, ValidationCriteria, ValidationOperator};
+        use crate::core::{DataValidationRule, ValidationCriteria, ValidationOperator};
 
         let mut sheet = Sheet::new("Test");
 
@@ -1952,7 +1952,7 @@ mod tests {
 
     #[test]
     fn test_data_validation_cell_validation() {
-        use crate::{DataValidationRule, ValidationCriteria, ValidationOperator, ValidationResult};
+        use crate::core::{DataValidationRule, ValidationCriteria, ValidationOperator, ValidationResult};
 
         let mut sheet = Sheet::new("Test");
 
@@ -1984,7 +1984,7 @@ mod tests {
 
     #[test]
     fn test_data_validation_dropdown_items() {
-        use crate::{DataValidationRule, ValidationCriteria, ListSource};
+        use crate::core::{DataValidationRule, ValidationCriteria, ListSource};
 
         let mut sheet = Sheet::new("Test");
 
@@ -2013,7 +2013,7 @@ mod tests {
 
     #[test]
     fn test_data_validation_replace_rule() {
-        use crate::{DataValidationRule, ValidationCriteria, ValidationOperator};
+        use crate::core::{DataValidationRule, ValidationCriteria, ValidationOperator};
 
         let mut sheet = Sheet::new("Test");
 
diff --git a/crates/cclab-grid-core/src/spatial.rs b/crates/cclab-grid/src/core/spatial.rs
similarity index 99%
rename from crates/cclab-grid-core/src/spatial.rs
rename to crates/cclab-grid/src/core/spatial.rs
index 63541103..64a6e2a7 100644
--- a/crates/cclab-grid-core/src/spatial.rs
+++ b/crates/cclab-grid/src/core/spatial.rs
@@ -1240,7 +1240,7 @@ fn spread_bits(mut x: u16) -> u16 {
 /// # Examples
 ///
 /// ```
-/// use cclab_grid_core::spatial::morton_encode;
+/// use cclab_grid::core::spatial::morton_encode;
 ///
 /// assert_eq!(morton_encode(0, 0), 0);
 /// assert_eq!(morton_encode(0, 1), 1);
diff --git a/crates/cclab-grid-core/src/state/clipboard.rs b/crates/cclab-grid/src/core/state/clipboard.rs
similarity index 100%
rename from crates/cclab-grid-core/src/state/clipboard.rs
rename to crates/cclab-grid/src/core/state/clipboard.rs
diff --git a/crates/cclab-grid-core/src/state/edit.rs b/crates/cclab-grid/src/core/state/edit.rs
similarity index 100%
rename from crates/cclab-grid-core/src/state/edit.rs
rename to crates/cclab-grid/src/core/state/edit.rs
diff --git a/crates/cclab-grid-core/src/state/input.rs b/crates/cclab-grid/src/core/state/input.rs
similarity index 100%
rename from crates/cclab-grid-core/src/state/input.rs
rename to crates/cclab-grid/src/core/state/input.rs
diff --git a/crates/cclab-grid-core/src/state/mod.rs b/crates/cclab-grid/src/core/state/mod.rs
similarity index 100%
rename from crates/cclab-grid-core/src/state/mod.rs
rename to crates/cclab-grid/src/core/state/mod.rs
diff --git a/crates/cclab-grid-core/src/state/selection.rs b/crates/cclab-grid/src/core/state/selection.rs
similarity index 100%
rename from crates/cclab-grid-core/src/state/selection.rs
rename to crates/cclab-grid/src/core/state/selection.rs
diff --git a/crates/cclab-grid-core/src/state/viewport.rs b/crates/cclab-grid/src/core/state/viewport.rs
similarity index 100%
rename from crates/cclab-grid-core/src/state/viewport.rs
rename to crates/cclab-grid/src/core/state/viewport.rs
diff --git a/crates/cclab-grid-core/src/validation.rs b/crates/cclab-grid/src/core/validation.rs
similarity index 99%
rename from crates/cclab-grid-core/src/validation.rs
rename to crates/cclab-grid/src/core/validation.rs
index 33eeed4d..22b6402d 100644
--- a/crates/cclab-grid-core/src/validation.rs
+++ b/crates/cclab-grid/src/core/validation.rs
@@ -1,4 +1,4 @@
-use crate::{CellValue, CellRange};
+use crate::core::{CellValue, CellRange};
 use serde::{Deserialize, Serialize};
 
 /// Comparison operators for validation
@@ -315,7 +315,7 @@ impl DataValidationRule {
 #[cfg(test)]
 mod tests {
     use super::*;
-    use crate::CellCoord;
+    use crate::core::CellCoord;
 
     #[test]
     fn test_list_validation_static() {
diff --git a/crates/cclab-grid-core/src/workbook.rs b/crates/cclab-grid/src/core/workbook.rs
similarity index 99%
rename from crates/cclab-grid-core/src/workbook.rs
rename to crates/cclab-grid/src/core/workbook.rs
index afc8a444..3fc80c2b 100644
--- a/crates/cclab-grid-core/src/workbook.rs
+++ b/crates/cclab-grid/src/core/workbook.rs
@@ -1,7 +1,7 @@
 use serde::{Deserialize, Serialize};
 
-use crate::sheet::Sheet;
-use crate::error::RusheetError;
+use crate::core::sheet::Sheet;
+use crate::core::error::RusheetError;
 
 /// Metadata about the workbook
 #[derive(Debug, Clone, Default, Serialize, Deserialize)]
diff --git a/crates/cclab-grid-db/src/lib.rs b/crates/cclab-grid/src/db/mod.rs
similarity index 81%
rename from crates/cclab-grid-db/src/lib.rs
rename to crates/cclab-grid/src/db/mod.rs
index b5688e29..5bf7b3eb 100644
--- a/crates/cclab-grid-db/src/lib.rs
+++ b/crates/cclab-grid/src/db/mod.rs
@@ -1,10 +1,10 @@
-//! # cclab-grid-db
+//! # cclab-grid db module
 //!
 //! High-performance spreadsheet database layer with Morton encoding and WAL-backed persistence.
 //!
 //! ## Architecture
 //!
-//! This crate provides:
+//! This module provides:
 //! - **Storage Layer**: Efficient cell storage using Morton encoding (Z-order curve)
 //! - **Query Layer**: Range queries and spatial queries for spreadsheet operations
 //! - **Snapshot Layer**: yrs (Yjs) snapshot storage for collaborative editing
@@ -14,21 +14,6 @@
 //! - `storage`: Cell storage engine with WAL support
 //! - `query`: Query builders for range and spatial queries
 //! - `snapshot`: yrs update/snapshot persistence
-//!
-//! ## Example
-//!
-//! ```rust,ignore
-//! use cclab_grid_db::storage::CellStore;
-//!
-//! // Create cell store
-//! let store = CellStore::new("./data", "sheet-1").await?;
-//!
-//! // Set a cell
-//! store.set_cell(0, 0, CellValue::Number(42.0)).await?;
-//!
-//! // Query cells in range
-//! let cells = store.query_range(0, 0, 10, 10).await?;
-//! ```
 
 pub mod query;
 pub mod snapshot;
diff --git a/crates/cclab-grid-db/src/query/mod.rs b/crates/cclab-grid/src/db/query/mod.rs
similarity index 100%
rename from crates/cclab-grid-db/src/query/mod.rs
rename to crates/cclab-grid/src/db/query/mod.rs
diff --git a/crates/cclab-grid-db/src/query/range.rs b/crates/cclab-grid/src/db/query/range.rs
similarity index 98%
rename from crates/cclab-grid-db/src/query/range.rs
rename to crates/cclab-grid/src/db/query/range.rs
index d1030f21..c7195280 100644
--- a/crates/cclab-grid-db/src/query/range.rs
+++ b/crates/cclab-grid/src/db/query/range.rs
@@ -2,8 +2,8 @@
 //!
 //! Provides efficient querying of cells in rectangular ranges.
 
-use crate::storage::StoredCell;
-use crate::{Result, SheetDbError};
+use crate::db::storage::StoredCell;
+use crate::db::{Result, SheetDbError};
 use serde::{Deserialize, Serialize};
 
 /// Query builder for rectangular cell ranges
diff --git a/crates/cclab-grid-db/src/query/spatial.rs b/crates/cclab-grid/src/db/query/spatial.rs
similarity index 98%
rename from crates/cclab-grid-db/src/query/spatial.rs
rename to crates/cclab-grid/src/db/query/spatial.rs
index c6f96860..e06252d1 100644
--- a/crates/cclab-grid-db/src/query/spatial.rs
+++ b/crates/cclab-grid/src/db/query/spatial.rs
@@ -5,8 +5,8 @@
 //! - Cluster detection
 //! - Sparse region identification
 
-use crate::storage::StoredCell;
-use crate::Result;
+use crate::db::storage::StoredCell;
+use crate::db::Result;
 use serde::{Deserialize, Serialize};
 
 /// Spatial query builder
diff --git a/crates/cclab-grid-db/src/snapshot/mod.rs b/crates/cclab-grid/src/db/snapshot/mod.rs
similarity index 100%
rename from crates/cclab-grid-db/src/snapshot/mod.rs
rename to crates/cclab-grid/src/db/snapshot/mod.rs
diff --git a/crates/cclab-grid-db/src/snapshot/store.rs b/crates/cclab-grid/src/db/snapshot/store.rs
similarity index 99%
rename from crates/cclab-grid-db/src/snapshot/store.rs
rename to crates/cclab-grid/src/db/snapshot/store.rs
index 97c6563c..73e12529 100644
--- a/crates/cclab-grid-db/src/snapshot/store.rs
+++ b/crates/cclab-grid/src/db/snapshot/store.rs
@@ -1,6 +1,6 @@
 //! yrs snapshot storage implementation
 
-use crate::Result;
+use crate::db::Result;
 use parking_lot::RwLock;
 use serde::{Deserialize, Serialize};
 use std::collections::HashMap;
diff --git a/crates/cclab-grid-db/src/storage/cell_store.rs b/crates/cclab-grid/src/db/storage/cell_store.rs
similarity index 98%
rename from crates/cclab-grid-db/src/storage/cell_store.rs
rename to crates/cclab-grid/src/db/storage/cell_store.rs
index 50b3fb1d..e7cd8427 100644
--- a/crates/cclab-grid-db/src/storage/cell_store.rs
+++ b/crates/cclab-grid/src/db/storage/cell_store.rs
@@ -2,10 +2,10 @@
 //!
 //! Provides efficient storage and retrieval of spreadsheet cells using Morton encoding.
 
-use crate::storage::morton::MortonKey;
-use crate::storage::wal::{GridWalOp, GridWalReader, GridWalWriter};
-use crate::Result;
-use cclab_grid_core::CellValue;
+use crate::db::storage::morton::MortonKey;
+use crate::db::storage::wal::{GridWalOp, GridWalReader, GridWalWriter};
+use crate::db::Result;
+use crate::core::CellValue;
 use parking_lot::RwLock;
 use serde::{Deserialize, Serialize};
 use std::collections::BTreeMap;
diff --git a/crates/cclab-grid-db/src/storage/mod.rs b/crates/cclab-grid/src/db/storage/mod.rs
similarity index 100%
rename from crates/cclab-grid-db/src/storage/mod.rs
rename to crates/cclab-grid/src/db/storage/mod.rs
diff --git a/crates/cclab-grid-db/src/storage/morton.rs b/crates/cclab-grid/src/db/storage/morton.rs
similarity index 99%
rename from crates/cclab-grid-db/src/storage/morton.rs
rename to crates/cclab-grid/src/db/storage/morton.rs
index 49244a9b..9f4c86b0 100644
--- a/crates/cclab-grid-db/src/storage/morton.rs
+++ b/crates/cclab-grid/src/db/storage/morton.rs
@@ -13,7 +13,7 @@ use serde::{Deserialize, Serialize};
 /// # Example
 ///
 /// ```rust
-/// use cclab_grid_db::storage::MortonKey;
+/// use cclab_grid::db::storage::MortonKey;
 ///
 /// let key = MortonKey::encode(10, 20);
 /// let (row, col) = key.decode();
diff --git a/crates/cclab-grid-db/src/storage/wal.rs b/crates/cclab-grid/src/db/storage/wal.rs
similarity index 99%
rename from crates/cclab-grid-db/src/storage/wal.rs
rename to crates/cclab-grid/src/db/storage/wal.rs
index 8064e382..207e0ca8 100644
--- a/crates/cclab-grid-db/src/storage/wal.rs
+++ b/crates/cclab-grid/src/db/storage/wal.rs
@@ -2,8 +2,8 @@
 //!
 //! Uses the shared cclab-wal crate with grid-specific operation types.
 
-use crate::Result;
-use cclab_grid_core::CellValue;
+use crate::db::Result;
+use crate::core::CellValue;
 use cclab_wal::{WalConfig, WalReader, WalWriter};
 use serde::{Deserialize, Serialize};
 use std::path::{Path, PathBuf};
diff --git a/crates/cclab-grid-formula/src/ast.rs b/crates/cclab-grid/src/formula/ast.rs
similarity index 98%
rename from crates/cclab-grid-formula/src/ast.rs
rename to crates/cclab-grid/src/formula/ast.rs
index 1a45e0a6..b256dfb4 100644
--- a/crates/cclab-grid-formula/src/ast.rs
+++ b/crates/cclab-grid/src/formula/ast.rs
@@ -1,4 +1,4 @@
-use cclab_grid_core::CellError;
+use crate::core::CellError;
 
 /// Abstract Syntax Tree for formula expressions
 #[derive(Debug, Clone, PartialEq)]
@@ -169,7 +169,7 @@ impl std::fmt::Display for Expr {
                 abs_col,
                 abs_row,
             } => {
-                use cclab_grid_core::col_to_label;
+                use crate::core::col_to_label;
                 let col_str = col_to_label(*col);
                 let row_str = row + 1; // Convert back to 1-indexed
                 write!(
diff --git a/crates/cclab-grid-formula/src/dependency.rs b/crates/cclab-grid/src/formula/dependency.rs
similarity index 99%
rename from crates/cclab-grid-formula/src/dependency.rs
rename to crates/cclab-grid/src/formula/dependency.rs
index 5108b6e9..49eae3be 100644
--- a/crates/cclab-grid-formula/src/dependency.rs
+++ b/crates/cclab-grid/src/formula/dependency.rs
@@ -1,6 +1,6 @@
 use std::collections::{HashMap, HashSet, VecDeque};
 
-use cclab_grid_core::CellError;
+use crate::core::CellError;
 
 /// Coordinates for a cell (row, col)
 pub type CellCoord = (u32, u32);
diff --git a/crates/cclab-grid-formula/src/evaluator.rs b/crates/cclab-grid/src/formula/evaluator.rs
similarity index 99%
rename from crates/cclab-grid-formula/src/evaluator.rs
rename to crates/cclab-grid/src/formula/evaluator.rs
index 3c63031a..b44c8766 100644
--- a/crates/cclab-grid-formula/src/evaluator.rs
+++ b/crates/cclab-grid/src/formula/evaluator.rs
@@ -1,6 +1,6 @@
-use crate::ast::{BinaryOp, Expr, UnaryOp};
-use crate::functions;
-use cclab_grid_core::{CellError, CellValue};
+use crate::formula::ast::{BinaryOp, Expr, UnaryOp};
+use crate::formula::functions;
+use crate::core::{CellError, CellValue};
 
 /// Evaluator for formula AST
 pub struct Evaluator<F>
@@ -869,8 +869,8 @@ where
 #[cfg(test)]
 mod tests {
     use super::*;
-    use crate::lexer::Lexer;
-    use crate::parser::Parser;
+    use crate::formula::lexer::Lexer;
+    use crate::formula::parser::Parser;
 
     fn eval(input: &str) -> CellValue {
         let mut lexer = Lexer::new(input);
@@ -1003,7 +1003,7 @@ mod tests {
     where
         F: Fn(Option<&str>, u32, u32) -> CellValue,
     {
-        use crate::parser_nom::NomParser;
+        use crate::formula::parser_nom::NomParser;
         let parser = NomParser::new();
         let ast = parser.parse(input).unwrap();
 
diff --git a/crates/cclab-grid-formula/src/functions/datetime.rs b/crates/cclab-grid/src/formula/functions/datetime.rs
similarity index 99%
rename from crates/cclab-grid-formula/src/functions/datetime.rs
rename to crates/cclab-grid/src/formula/functions/datetime.rs
index f1a6f092..db75451c 100644
--- a/crates/cclab-grid-formula/src/functions/datetime.rs
+++ b/crates/cclab-grid/src/formula/functions/datetime.rs
@@ -1,5 +1,5 @@
-use cclab_grid_core::cell::CellValue;
-use cclab_grid_core::error::CellError;
+use crate::core::cell::CellValue;
+use crate::core::error::CellError;
 
 // Excel date constants
 const EXCEL_EPOCH_YEAR: i32 = 1900;
diff --git a/crates/cclab-grid-formula/src/functions/logical.rs b/crates/cclab-grid/src/formula/functions/logical.rs
similarity index 98%
rename from crates/cclab-grid-formula/src/functions/logical.rs
rename to crates/cclab-grid/src/formula/functions/logical.rs
index 695d8ec4..b8a010dc 100644
--- a/crates/cclab-grid-formula/src/functions/logical.rs
+++ b/crates/cclab-grid/src/formula/functions/logical.rs
@@ -1,4 +1,4 @@
-use cclab_grid_core::{CellError, CellValue};
+use crate::core::{CellError, CellValue};
 
 /// IF - Conditional evaluation
 pub fn if_fn(values: &[CellValue]) -> CellValue {
diff --git a/crates/cclab-grid-formula/src/functions/lookup.rs b/crates/cclab-grid/src/formula/functions/lookup.rs
similarity index 99%
rename from crates/cclab-grid-formula/src/functions/lookup.rs
rename to crates/cclab-grid/src/formula/functions/lookup.rs
index 4fa05bea..5287f8b0 100644
--- a/crates/cclab-grid-formula/src/functions/lookup.rs
+++ b/crates/cclab-grid/src/formula/functions/lookup.rs
@@ -1,4 +1,4 @@
-use cclab_grid_core::{CellError, CellValue};
+use crate::core::{CellError, CellValue};
 
 /// MATCH - Search for a value in an array and return its relative position
 /// Args: lookup_value, lookup_array (slice), match_type
diff --git a/crates/cclab-grid-formula/src/functions/math.rs b/crates/cclab-grid/src/formula/functions/math.rs
similarity index 99%
rename from crates/cclab-grid-formula/src/functions/math.rs
rename to crates/cclab-grid/src/formula/functions/math.rs
index d86438db..ba536803 100644
--- a/crates/cclab-grid-formula/src/functions/math.rs
+++ b/crates/cclab-grid/src/formula/functions/math.rs
@@ -1,4 +1,4 @@
-use cclab_grid_core::{CellError, CellValue};
+use crate::core::{CellError, CellValue};
 
 /// SUM - Sum all numeric values
 pub fn sum(values: &[CellValue]) -> CellValue {
diff --git a/crates/cclab-grid-formula/src/functions/mod.rs b/crates/cclab-grid/src/formula/functions/mod.rs
similarity index 100%
rename from crates/cclab-grid-formula/src/functions/mod.rs
rename to crates/cclab-grid/src/formula/functions/mod.rs
diff --git a/crates/cclab-grid-formula/src/functions/text.rs b/crates/cclab-grid/src/formula/functions/text.rs
similarity index 99%
rename from crates/cclab-grid-formula/src/functions/text.rs
rename to crates/cclab-grid/src/formula/functions/text.rs
index 1c7e1d76..398801cc 100644
--- a/crates/cclab-grid-formula/src/functions/text.rs
+++ b/crates/cclab-grid/src/formula/functions/text.rs
@@ -1,4 +1,4 @@
-use cclab_grid_core::{CellError, CellValue};
+use crate::core::{CellError, CellValue};
 
 /// CONCAT / CONCATENATE - Concatenate strings
 pub fn concat(values: &[CellValue]) -> CellValue {
diff --git a/crates/cclab-grid-formula/src/lexer.rs b/crates/cclab-grid/src/formula/lexer.rs
similarity index 99%
rename from crates/cclab-grid-formula/src/lexer.rs
rename to crates/cclab-grid/src/formula/lexer.rs
index 56835461..2da6ce66 100644
--- a/crates/cclab-grid-formula/src/lexer.rs
+++ b/crates/cclab-grid/src/formula/lexer.rs
@@ -1,4 +1,4 @@
-use cclab_grid_core::CellError;
+use crate::core::CellError;
 
 /// Token types for formula parsing
 #[derive(Debug, Clone, PartialEq)]
diff --git a/crates/cclab-grid-formula/src/lib.rs b/crates/cclab-grid/src/formula/mod.rs
similarity index 99%
rename from crates/cclab-grid-formula/src/lib.rs
rename to crates/cclab-grid/src/formula/mod.rs
index 340a63a9..68cff53f 100644
--- a/crates/cclab-grid-formula/src/lib.rs
+++ b/crates/cclab-grid/src/formula/mod.rs
@@ -15,7 +15,7 @@ pub use parser::Parser;
 pub use parser_nom::NomParser;
 pub use reference_shifter::{shift_formula_cols, shift_formula_rows};
 
-use cclab_grid_core::{CellError, CellValue};
+use crate::core::{CellError, CellValue};
 
 /// Parse and evaluate a formula expression
 ///
diff --git a/crates/cclab-grid-formula/src/parser.rs b/crates/cclab-grid/src/formula/parser.rs
similarity index 98%
rename from crates/cclab-grid-formula/src/parser.rs
rename to crates/cclab-grid/src/formula/parser.rs
index efaa7faa..8615cb82 100644
--- a/crates/cclab-grid-formula/src/parser.rs
+++ b/crates/cclab-grid/src/formula/parser.rs
@@ -1,6 +1,6 @@
-use crate::ast::{BinaryOp, Expr, UnaryOp};
-use crate::lexer::Token;
-use cclab_grid_core::col_from_label;
+use crate::formula::ast::{BinaryOp, Expr, UnaryOp};
+use crate::formula::lexer::Token;
+use crate::core::col_from_label;
 
 /// Parser for formula expressions
 pub struct Parser {
@@ -327,7 +327,7 @@ fn parse_cell_reference(ref_str: &str) -> Result<Expr, String> {
 #[cfg(test)]
 mod tests {
     use super::*;
-    use crate::lexer::Lexer;
+    use crate::formula::lexer::Lexer;
 
     fn parse(input: &str) -> Result<Expr, String> {
         let mut lexer = Lexer::new(input);
diff --git a/crates/cclab-grid-formula/src/parser_nom.rs b/crates/cclab-grid/src/formula/parser_nom.rs
similarity index 99%
rename from crates/cclab-grid-formula/src/parser_nom.rs
rename to crates/cclab-grid/src/formula/parser_nom.rs
index bfcaaa4a..3b5e1ef0 100644
--- a/crates/cclab-grid-formula/src/parser_nom.rs
+++ b/crates/cclab-grid/src/formula/parser_nom.rs
@@ -13,8 +13,8 @@ use nom::{
     IResult,
 };
 
-use crate::ast::{BinaryOp, Expr, UnaryOp};
-use cclab_grid_core::CellError;
+use crate::formula::ast::{BinaryOp, Expr, UnaryOp};
+use crate::core::CellError;
 
 // =============================================================================
 // Error Type
diff --git a/crates/cclab-grid-formula/src/reference_shifter.rs b/crates/cclab-grid/src/formula/reference_shifter.rs
similarity index 97%
rename from crates/cclab-grid-formula/src/reference_shifter.rs
rename to crates/cclab-grid/src/formula/reference_shifter.rs
index b7f8818a..93d2cf18 100644
--- a/crates/cclab-grid-formula/src/reference_shifter.rs
+++ b/crates/cclab-grid/src/formula/reference_shifter.rs
@@ -1,5 +1,5 @@
-use crate::ast::Expr;
-use crate::parser_nom::NomParser;
+use crate::formula::ast::Expr;
+use crate::formula::parser_nom::NomParser;
 
 /// Shift formula references when rows are inserted/deleted.
 ///
@@ -15,7 +15,7 @@ use crate::parser_nom::NomParser;
 ///
 /// Insert rows:
 /// ```
-/// use cclab_grid_formula::shift_formula_rows;
+/// use cclab_grid::formula::shift_formula_rows;
 ///
 /// let formula = "=A1+B3";
 /// let result = shift_formula_rows(formula, 2, 2);
@@ -24,7 +24,7 @@ use crate::parser_nom::NomParser;
 ///
 /// Delete rows:
 /// ```
-/// use cclab_grid_formula::shift_formula_rows;
+/// use cclab_grid::formula::shift_formula_rows;
 ///
 /// let formula = "=A1+B3";
 /// let result = shift_formula_rows(formula, 1, -1);
@@ -33,7 +33,7 @@ use crate::parser_nom::NomParser;
 ///
 /// Absolute references don't shift:
 /// ```
-/// use cclab_grid_formula::shift_formula_rows;
+/// use cclab_grid::formula::shift_formula_rows;
 ///
 /// let formula = "=$A$1+B3";
 /// let result = shift_formula_rows(formula, 0, 2);
@@ -42,7 +42,7 @@ use crate::parser_nom::NomParser;
 ///
 /// Deleted reference returns None:
 /// ```
-/// use cclab_grid_formula::shift_formula_rows;
+/// use cclab_grid::formula::shift_formula_rows;
 ///
 /// let formula = "=A1+B3";
 /// let result = shift_formula_rows(formula, 2, -2);
@@ -74,7 +74,7 @@ pub fn shift_formula_rows(formula: &str, at_row: u32, delta: i32) -> Option<Stri
 ///
 /// Insert columns:
 /// ```
-/// use cclab_grid_formula::shift_formula_cols;
+/// use cclab_grid::formula::shift_formula_cols;
 ///
 /// let formula = "=A1+B1";
 /// let result = shift_formula_cols(formula, 1, 2);
@@ -83,7 +83,7 @@ pub fn shift_formula_rows(formula: &str, at_row: u32, delta: i32) -> Option<Stri
 ///
 /// Delete columns:
 /// ```
-/// use cclab_grid_formula::shift_formula_cols;
+/// use cclab_grid::formula::shift_formula_cols;
 ///
 /// let formula = "=A1+C1";
 /// let result = shift_formula_cols(formula, 1, -1);
@@ -92,7 +92,7 @@ pub fn shift_formula_rows(formula: &str, at_row: u32, delta: i32) -> Option<Stri
 ///
 /// Absolute column references don't shift:
 /// ```
-/// use cclab_grid_formula::shift_formula_cols;
+/// use cclab_grid::formula::shift_formula_cols;
 ///
 /// let formula = "=$A1+B1";
 /// let result = shift_formula_cols(formula, 0, 2);
diff --git a/crates/cclab-grid-history/src/command.rs b/crates/cclab-grid/src/history/command.rs
similarity index 98%
rename from crates/cclab-grid-history/src/command.rs
rename to crates/cclab-grid/src/history/command.rs
index e9b3336e..3e142074 100644
--- a/crates/cclab-grid-history/src/command.rs
+++ b/crates/cclab-grid/src/history/command.rs
@@ -1,6 +1,6 @@
-use cclab_grid_core::{Cell, CellContent, CellCoord, CellFormat, CellRange, CellValue, Sheet};
-use cclab_grid_core::sheet::FilterState;
-use cclab_grid_formula::{shift_formula_rows, shift_formula_cols};
+use crate::core::{Cell, CellContent, CellCoord, CellFormat, CellRange, CellValue, Sheet};
+use crate::core::sheet::FilterState;
+use crate::formula::{shift_formula_rows, shift_formula_cols};
 use std::collections::HashSet;
 
 /// Type alias for boxed commands
@@ -54,7 +54,7 @@ impl SetCellValueCommand {
     }
 
     pub fn from_input(coord: CellCoord, input: &str) -> Self {
-        let content = cclab_grid_core::sheet::parse_cell_input(input);
+        let content = crate::core::sheet::parse_cell_input(input);
         Self::new(coord, content)
     }
 }
@@ -379,8 +379,8 @@ impl Command for InsertRowsCommand {
             if let Some(cell) = sheet.get_cell(coord) {
                 if let CellContent::Formula { expression, .. } = &cell.content {
                     // Try to shift the formula
-                    if let Some(new_formula) = shift_formula_rows(expression, self.at_row, self.count as i32) {
-                        if &new_formula != expression {
+                    if let Some(new_formula) = shift_formula_rows(&expression, self.at_row, self.count as i32) {
+                        if new_formula != *expression {
                             self.formula_updates.push((coord, expression.clone(), new_formula));
                         }
                     }
@@ -493,8 +493,8 @@ impl Command for DeleteRowsCommand {
             if let Some(cell) = sheet.get_cell(coord) {
                 if let CellContent::Formula { expression, .. } = &cell.content {
                     // Try to shift the formula (negative delta for deletion)
-                    if let Some(new_formula) = shift_formula_rows(expression, self.at_row, -(self.count as i32)) {
-                        if &new_formula != expression {
+                    if let Some(new_formula) = shift_formula_rows(&expression, self.at_row, -(self.count as i32)) {
+                        if new_formula != *expression {
                             self.formula_updates.push((coord, expression.clone(), new_formula));
                         }
                     } else {
@@ -638,8 +638,8 @@ impl Command for InsertColsCommand {
             if let Some(cell) = sheet.get_cell(coord) {
                 if let CellContent::Formula { expression, .. } = &cell.content {
                     // Try to shift the formula
-                    if let Some(new_formula) = shift_formula_cols(expression, self.at_col, self.count as i32) {
-                        if &new_formula != expression {
+                    if let Some(new_formula) = shift_formula_cols(&expression, self.at_col, self.count as i32) {
+                        if new_formula != *expression {
                             self.formula_updates.push((coord, expression.clone(), new_formula));
                         }
                     }
@@ -752,8 +752,8 @@ impl Command for DeleteColsCommand {
             if let Some(cell) = sheet.get_cell(coord) {
                 if let CellContent::Formula { expression, .. } = &cell.content {
                     // Try to shift the formula (negative delta for deletion)
-                    if let Some(new_formula) = shift_formula_cols(expression, self.at_col, -(self.count as i32)) {
-                        if &new_formula != expression {
+                    if let Some(new_formula) = shift_formula_cols(&expression, self.at_col, -(self.count as i32)) {
+                        if new_formula != *expression {
                             self.formula_updates.push((coord, expression.clone(), new_formula));
                         }
                     } else {
diff --git a/crates/cclab-grid-history/src/lib.rs b/crates/cclab-grid/src/history/mod.rs
similarity index 100%
rename from crates/cclab-grid-history/src/lib.rs
rename to crates/cclab-grid/src/history/mod.rs
diff --git a/crates/cclab-grid-history/src/stack.rs b/crates/cclab-grid/src/history/stack.rs
similarity index 97%
rename from crates/cclab-grid-history/src/stack.rs
rename to crates/cclab-grid/src/history/stack.rs
index 1a6f206e..b9a1fab5 100644
--- a/crates/cclab-grid-history/src/stack.rs
+++ b/crates/cclab-grid/src/history/stack.rs
@@ -1,5 +1,5 @@
-use crate::command::CommandBox;
-use cclab_grid_core::{CellCoord, Sheet};
+use crate::history::command::CommandBox;
+use crate::core::{CellCoord, Sheet};
 
 /// Manages undo/redo history for spreadsheet operations
 #[derive(Default)]
@@ -136,8 +136,8 @@ impl std::fmt::Debug for HistoryManager {
 #[cfg(test)]
 mod tests {
     use super::*;
-    use crate::command::SetCellValueCommand;
-    use cclab_grid_core::CellValue;
+    use crate::history::command::SetCellValueCommand;
+    use crate::core::CellValue;
 
     #[test]
     fn test_undo_redo() {
diff --git a/crates/cclab-grid/src/lib.rs b/crates/cclab-grid/src/lib.rs
new file mode 100644
index 00000000..e01eac49
--- /dev/null
+++ b/crates/cclab-grid/src/lib.rs
@@ -0,0 +1,23 @@
+// @spec .score/changes/grid-consolidate/groups/consolidate-grid-crates/specs/grid-crate-structure.md#R1
+//! # cclab-grid
+//!
+//! Unified spreadsheet engine crate combining core data structures, formula evaluation,
+//! undo/redo history, database persistence, and collaboration server.
+//!
+//! ## Modules
+//!
+//! - `core` - Sparse matrix storage, cell types, coordinate system, range operations, sheet management
+//! - `formula` - Formula parser, evaluator, function library, dependency graph
+//! - `history` - Command-based undo/redo history
+//! - `db` - Morton encoding persistence, WAL-backed storage, range queries (feature: `db`)
+//! - `server` - Axum web server, CRDT collaboration, WebSocket handlers (feature: `server`)
+
+pub mod core;
+pub mod formula;
+pub mod history;
+
+#[cfg(feature = "db")]
+pub mod db;
+
+#[cfg(feature = "server")]
+pub mod server;
diff --git a/crates/cclab-grid-server/src/api/health.rs b/crates/cclab-grid/src/server/api/health.rs
similarity index 93%
rename from crates/cclab-grid-server/src/api/health.rs
rename to crates/cclab-grid/src/server/api/health.rs
index efd2f402..d06158c4 100644
--- a/crates/cclab-grid-server/src/api/health.rs
+++ b/crates/cclab-grid/src/server/api/health.rs
@@ -1,7 +1,7 @@
 use axum::{routing::get, Json, Router};
 use serde::Serialize;
 
-use crate::AppState;
+use crate::server::AppState;
 
 #[derive(Serialize)]
 struct HealthResponse {
diff --git a/crates/cclab-grid-server/src/api/mod.rs b/crates/cclab-grid/src/server/api/mod.rs
similarity index 87%
rename from crates/cclab-grid-server/src/api/mod.rs
rename to crates/cclab-grid/src/server/api/mod.rs
index b54d5f48..1c873aa5 100644
--- a/crates/cclab-grid-server/src/api/mod.rs
+++ b/crates/cclab-grid/src/server/api/mod.rs
@@ -3,7 +3,7 @@ mod workbooks;
 
 use axum::Router;
 
-use crate::AppState;
+use crate::server::AppState;
 
 /// Create the API router
 pub fn router() -> Router<AppState> {
diff --git a/crates/cclab-grid-server/src/api/workbooks.rs b/crates/cclab-grid/src/server/api/workbooks.rs
similarity index 97%
rename from crates/cclab-grid-server/src/api/workbooks.rs
rename to crates/cclab-grid/src/server/api/workbooks.rs
index 4bb13597..0c61f7dd 100644
--- a/crates/cclab-grid-server/src/api/workbooks.rs
+++ b/crates/cclab-grid/src/server/api/workbooks.rs
@@ -6,9 +6,9 @@ use axum::{
 use serde::{Deserialize, Serialize};
 use uuid::Uuid;
 
-use crate::db::models::Workbook;
-use crate::error::AppError;
-use crate::AppState;
+use crate::server::db::models::Workbook;
+use crate::server::error::AppError;
+use crate::server::AppState;
 
 /// Request to create a new workbook
 #[derive(Debug, Deserialize)]
diff --git a/crates/cclab-grid-server/src/collab/document.rs b/crates/cclab-grid/src/server/collab/document.rs
similarity index 100%
rename from crates/cclab-grid-server/src/collab/document.rs
rename to crates/cclab-grid/src/server/collab/document.rs
diff --git a/crates/cclab-grid-server/src/collab/mod.rs b/crates/cclab-grid/src/server/collab/mod.rs
similarity index 100%
rename from crates/cclab-grid-server/src/collab/mod.rs
rename to crates/cclab-grid/src/server/collab/mod.rs
diff --git a/crates/cclab-grid-server/src/collab/websocket.rs b/crates/cclab-grid/src/server/collab/websocket.rs
similarity index 98%
rename from crates/cclab-grid-server/src/collab/websocket.rs
rename to crates/cclab-grid/src/server/collab/websocket.rs
index 0c56b02b..8c46c9f4 100644
--- a/crates/cclab-grid-server/src/collab/websocket.rs
+++ b/crates/cclab-grid/src/server/collab/websocket.rs
@@ -10,7 +10,7 @@ use axum::{
 use futures::{SinkExt, StreamExt};
 use uuid::Uuid;
 
-use crate::AppState;
+use crate::server::AppState;
 
 /// WebSocket handler for collaboration
 async fn ws_handler(
diff --git a/crates/cclab-grid-server/src/config.rs b/crates/cclab-grid/src/server/config.rs
similarity index 100%
rename from crates/cclab-grid-server/src/config.rs
rename to crates/cclab-grid/src/server/config.rs
diff --git a/crates/cclab-grid-server/src/db/mod.rs b/crates/cclab-grid/src/server/db/mod.rs
similarity index 99%
rename from crates/cclab-grid-server/src/db/mod.rs
rename to crates/cclab-grid/src/server/db/mod.rs
index 22b0c6b6..aa6c3a10 100644
--- a/crates/cclab-grid-server/src/db/mod.rs
+++ b/crates/cclab-grid/src/server/db/mod.rs
@@ -7,8 +7,8 @@ use tokio::sync::RwLock;
 use uuid::Uuid;
 use chrono::Utc;
 
-use crate::error::AppError;
-use cclab_grid_db::{CellStore, YrsStore, SheetDbError};
+use crate::server::error::AppError;
+use crate::db::{CellStore, YrsStore, SheetDbError};
 use models::Workbook;
 
 /// Database connection wrapper
diff --git a/crates/cclab-grid-server/src/db/models.rs b/crates/cclab-grid/src/server/db/models.rs
similarity index 100%
rename from crates/cclab-grid-server/src/db/models.rs
rename to crates/cclab-grid/src/server/db/models.rs
diff --git a/crates/cclab-grid-server/src/error.rs b/crates/cclab-grid/src/server/error.rs
similarity index 100%
rename from crates/cclab-grid-server/src/error.rs
rename to crates/cclab-grid/src/server/error.rs
diff --git a/crates/cclab-grid-server/src/lib.rs b/crates/cclab-grid/src/server/mod.rs
similarity index 93%
rename from crates/cclab-grid-server/src/lib.rs
rename to crates/cclab-grid/src/server/mod.rs
index 3b6336b4..b3eb062a 100644
--- a/crates/cclab-grid-server/src/lib.rs
+++ b/crates/cclab-grid/src/server/mod.rs
@@ -10,9 +10,9 @@ use tokio::net::TcpListener;
 use tower_http::cors::{Any, CorsLayer};
 use tower_http::trace::TraceLayer;
 
-use crate::collab::DocumentStore;
-use crate::config::Config;
-use crate::db::Database;
+use crate::server::collab::DocumentStore;
+use crate::server::config::Config;
+use crate::server::db::Database;
 
 /// Application state shared across all handlers
 #[derive(Clone)]

```

## Review: grid-crate-structure

verdict: APPROVED
reviewer: reviewer
iteration: 1
change_id: grid-consolidate

**Summary**: Implementation fully matches the spec. All 5 grid crates consolidated into a single cclab-grid crate with proper feature gating and sub-module structure. All tests pass. No regressions.



## Alignment Warnings

7 violation(s) found across 1 spec(s).

| File | Kind | Message |
|------|------|---------|
| /Users/chris.cheng/cclab/main/.score/tech_design/crates/cclab-grid/logic/crate-structure.md | missing_section_annotation | Section 'Overview' at line 9 has no type annotation (expected <!-- type: X lang: Y -->) |
| /Users/chris.cheng/cclab/main/.score/tech_design/crates/cclab-grid/logic/crate-structure.md | missing_section_annotation | Section 'Diagrams' at line 42 has no type annotation (expected <!-- type: X lang: Y -->) |
| /Users/chris.cheng/cclab/main/.score/tech_design/crates/cclab-grid/logic/crate-structure.md | missing_section_annotation | Section 'API Spec' at line 64 has no type annotation (expected <!-- type: X lang: Y -->) |
| /Users/chris.cheng/cclab/main/.score/tech_design/crates/cclab-grid/logic/crate-structure.md | missing_section_annotation | Section 'Changes' at line 95 has no type annotation (expected <!-- type: X lang: Y -->) |
| /Users/chris.cheng/cclab/main/.score/tech_design/crates/cclab-grid/logic/crate-structure.md | format_priority_violation | Section 'Wireframe' (type: wireframe) requires a ```yaml code block but none found |
| /Users/chris.cheng/cclab/main/.score/tech_design/crates/cclab-grid/logic/crate-structure.md | format_priority_violation | Section 'Component' (type: component) requires a ```json code block but none found |
| /Users/chris.cheng/cclab/main/.score/tech_design/crates/cclab-grid/logic/crate-structure.md | format_priority_violation | Section 'Design Token' (type: design-token) requires a ```json code block but none found |
