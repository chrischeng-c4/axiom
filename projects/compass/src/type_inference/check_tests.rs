//! Tests for type checking

use super::*;
use crate::syntax::MultiParser;

fn check_code(code: &str) -> Vec<Diagnostic> {
    let mut parser = MultiParser::new().unwrap();
    let parsed = parser.parse(code, crate::syntax::Language::Python).unwrap();
    let mut checker = TypeChecker::new(code);
    checker.check_file(&parsed)
}

#[test]
fn test_type_mismatch() {
    let diagnostics = check_code(
        r#"
x: int = "hello"
"#,
    );

    assert!(!diagnostics.is_empty());
    assert!(diagnostics
        .iter()
        .any(|d| d.code == "TC001" && d.message.contains("Type mismatch")));
}

#[test]
fn test_compatible_assignment() {
    let diagnostics = check_code(
        r#"
x: int = 42
y: float = 3.14
z: float = 42  # int is assignable to float
"#,
    );

    // Should not have type mismatch errors (may have TC002 for missing return types)
    assert!(!diagnostics.iter().any(|d| d.code == "TC001"));
}

#[test]
fn test_no_type_error_for_correct_types() {
    // Simple cases that should not produce type errors
    let diagnostics = check_code(
        r#"
x: int = 42
y: str = "hello"
z: float = 3.14
"#,
    );

    // Should not have type mismatch errors
    assert!(!diagnostics.iter().any(|d| d.code == "TC001"));
}

#[test]
fn test_return_type_mismatch() {
    let diagnostics = check_code(
        r#"
def get_number() -> int:
    return "hello"
"#,
    );

    assert!(diagnostics
        .iter()
        .any(|d| d.code == "TC003" && d.message.contains("Incompatible return type")));
}

#[test]
fn test_return_type_correct() {
    let diagnostics = check_code(
        r#"
def get_number() -> int:
    return 42
"#,
    );

    // Should not have return type errors
    assert!(!diagnostics.iter().any(|d| d.code == "TC003"));
}

#[test]
fn test_function_missing_return() {
    let diagnostics = check_code(
        r#"
def get_number() -> int:
    x = 42
"#,
    );

    // Should warn about missing return
    assert!(diagnostics
        .iter()
        .any(|d| d.code == "TC003" && d.message.contains("may not return")));
}

#[test]
fn test_class_method_return_type() {
    let diagnostics = check_code(
        r#"
class Calculator:
    def add(self, x: int, y: int) -> int:
        return x + y
"#,
    );

    // Should not have return type errors - add returns int
    assert!(!diagnostics
        .iter()
        .any(|d| d.code == "TC003" && d.message.contains("Incompatible return type")));
}

#[test]
fn test_class_method_wrong_return() {
    let diagnostics = check_code(
        r#"
class Greeter:
    def greet(self) -> str:
        return 42
"#,
    );

    // Should have return type error - returns int instead of str
    assert!(diagnostics
        .iter()
        .any(|d| d.code == "TC003" && d.message.contains("Incompatible return type")));
}

#[test]
fn test_class_type_checking() {
    let diagnostics = check_code(
        r#"
class Point:
    def __init__(self, x: int, y: int) -> None:
        self.x = x
        self.y = y

p = Point(1, 2)
"#,
    );

    // Basic class definition should not have errors
    // (might have TC002 for missing __init__ return type hint but that's fine)
    assert!(!diagnostics.iter().any(|d| d.code == "TC001"));
}

#[test]
fn test_none_check_narrowing() {
    // Test that `if x is not None` properly narrows Optional[str] to str
    let diagnostics = check_code(
        r#"
def process(x: str | None) -> str:
    if x is not None:
        return x
    return "default"
"#,
    );

    // Should NOT have type errors - x is narrowed to str in the if branch
    assert!(!diagnostics
        .iter()
        .any(|d| d.code == "TC003" && d.message.contains("Incompatible")));
}

#[test]
fn test_if_statement_basic() {
    // Test basic if statement processing
    let diagnostics = check_code(
        r#"
def test(x: int) -> int:
    if x > 0:
        return x
    else:
        return 0
"#,
    );

    // Should not have errors
    assert!(!diagnostics
        .iter()
        .any(|d| d.code == "TC003" && d.message.contains("Incompatible")));
}

#[test]
fn test_import_statement() {
    // Test that import statements are processed without errors
    let diagnostics = check_code(
        r#"
from typing import List, Optional

def foo(items: List[int]) -> Optional[int]:
    if items:
        return items[0]
    return None
"#,
    );

    // Should not have type mismatch errors
    assert!(!diagnostics.iter().any(|d| d.code == "TC001"));
}

#[test]
fn test_import_module() {
    // Test regular import statement
    let diagnostics = check_code(
        r#"
import os

x = os
"#,
    );

    // Should not have errors
    assert!(!diagnostics.iter().any(|d| d.code == "TC001"));
}

#[test]
fn test_for_loop() {
    // Test for loop processes without errors
    let diagnostics = check_code(
        r#"
def process(items: list[int]) -> int:
    total = 0
    for item in items:
        total = total + item
    return total
"#,
    );

    // Should not have type errors
    assert!(!diagnostics
        .iter()
        .any(|d| d.code == "TC003" && d.message.contains("Incompatible")));
}

#[test]
fn test_while_loop() {
    // Test while loop processes without errors
    let diagnostics = check_code(
        r#"
def countdown(n: int) -> int:
    while n > 0:
        n = n - 1
    return n
"#,
    );

    // Should not have type errors
    assert!(!diagnostics
        .iter()
        .any(|d| d.code == "TC003" && d.message.contains("Incompatible")));
}

#[test]
fn test_try_except() {
    // Test try/except processes without errors
    let diagnostics = check_code(
        r#"
def safe_divide(a: int, b: int) -> int:
    try:
        return a // b
    except ZeroDivisionError:
        return 0
"#,
    );

    // Should not have type errors
    assert!(!diagnostics
        .iter()
        .any(|d| d.code == "TC003" && d.message.contains("Incompatible")));
}

#[test]
fn test_subclass_assignment() {
    // Test that subclass instances can be assigned to parent type
    let diagnostics = check_code(
        r#"
class Animal:
    def speak(self) -> str:
        return "sound"

class Dog(Animal):
    def speak(self) -> str:
        return "woof"

def make_sound(a: Animal) -> str:
    return a.speak()

d: Dog = Dog()
a: Animal = d
make_sound(d)
"#,
    );

    // Should not have type mismatch errors for subclass assignment
    assert!(
        !diagnostics.iter().any(|d| d.code == "TC001"),
        "Should allow subclass assignment to parent type"
    );
}
