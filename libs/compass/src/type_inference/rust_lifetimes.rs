//! Rust lifetime analysis
//!
//! This module provides lifetime analysis for Rust code, including:
//! - Lifetime constraint tracking
//! - Borrow checking (mutable vs immutable)
//! - Lifetime error reporting

use std::collections::{HashMap, HashSet};

use super::rust_types::{Lifetime, LifetimeId, RustType};

// ============================================================================
// Lifetime Constraints
// ============================================================================

/// A lifetime constraint representing 'a: 'b (a outlives b)
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct LifetimeConstraint {
    /// The longer lifetime (must outlive `shorter`)
    pub longer: LifetimeId,
    /// The shorter lifetime (must be outlived by `longer`)
    pub shorter: LifetimeId,
    /// Source location of the constraint
    pub span: Option<(usize, usize)>,
}

/// A borrow of a value
#[derive(Debug, Clone)]
pub struct Borrow {
    /// Unique identifier for this borrow
    pub id: BorrowId,
    /// The lifetime of the borrow
    pub lifetime: LifetimeId,
    /// Whether this is a mutable borrow
    pub is_mutable: bool,
    /// The path being borrowed (e.g., "x", "x.field", "*x")
    pub path: String,
    /// Source location of the borrow
    pub span: (usize, usize),
}

/// Unique identifier for borrows
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BorrowId(pub usize);

// ============================================================================
// Lifetime Errors
// ============================================================================

/// Lifetime-related error
#[derive(Debug, Clone)]
pub struct LifetimeError {
    /// Error message
    pub message: String,
    /// Error kind
    pub kind: LifetimeErrorKind,
    /// Source location
    pub span: Option<(usize, usize)>,
    /// Related locations (e.g., conflicting borrow locations)
    pub related_spans: Vec<(usize, usize)>,
}

/// Kind of lifetime error
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LifetimeErrorKind {
    /// Lifetime does not live long enough
    DoesNotLiveLongEnough,
    /// Conflicting borrows (e.g., mutable + immutable)
    ConflictingBorrows,
    /// Cannot move out of borrowed reference
    CannotMoveFromBorrow,
    /// Value used after move
    UseAfterMove,
    /// Borrow of moved value
    BorrowOfMovedValue,
    /// Cannot assign to immutable borrow
    CannotAssignToImmutable,
    /// Lifetime constraint not satisfied
    ConstraintNotSatisfied,
}

// ============================================================================
// Borrow State
// ============================================================================

/// Tracks active borrows in a scope
#[derive(Debug, Clone, Default)]
pub struct BorrowState {
    /// Active borrows by path
    borrows: HashMap<String, Vec<Borrow>>,
    /// Moved values
    moved: HashSet<String>,
    /// Borrow ID counter
    next_borrow_id: usize,
}

impl BorrowState {
    /// Create a new borrow state
    pub fn new() -> Self {
        Self::default()
    }

    /// Record a borrow
    pub fn borrow(
        &mut self,
        path: String,
        lifetime: LifetimeId,
        is_mutable: bool,
        span: (usize, usize),
    ) -> Result<BorrowId, LifetimeError> {
        // Check if value was moved
        if self.moved.contains(&path) {
            return Err(LifetimeError {
                message: format!("Cannot borrow `{}` after move", path),
                kind: LifetimeErrorKind::BorrowOfMovedValue,
                span: Some(span),
                related_spans: vec![],
            });
        }

        // Check for conflicting borrows
        if let Some(existing) = self.borrows.get(&path) {
            for borrow in existing {
                if is_mutable || borrow.is_mutable {
                    // Mutable borrow conflicts with any other borrow
                    return Err(LifetimeError {
                        message: format!(
                            "Cannot borrow `{}` as {} because it is already borrowed as {}",
                            path,
                            if is_mutable { "mutable" } else { "immutable" },
                            if borrow.is_mutable {
                                "mutable"
                            } else {
                                "immutable"
                            }
                        ),
                        kind: LifetimeErrorKind::ConflictingBorrows,
                        span: Some(span),
                        related_spans: vec![borrow.span],
                    });
                }
            }
        }

        let id = BorrowId(self.next_borrow_id);
        self.next_borrow_id += 1;

        let borrow = Borrow {
            id,
            lifetime,
            is_mutable,
            path: path.clone(),
            span,
        };

        self.borrows.entry(path).or_default().push(borrow);
        Ok(id)
    }

    /// End a borrow (when lifetime ends)
    pub fn end_borrow(&mut self, id: BorrowId) {
        for borrows in self.borrows.values_mut() {
            borrows.retain(|b| b.id != id);
        }
    }

    /// Record a move
    pub fn move_value(&mut self, path: &str, span: (usize, usize)) -> Result<(), LifetimeError> {
        // Check if there are active borrows
        if let Some(borrows) = self.borrows.get(path) {
            if !borrows.is_empty() {
                return Err(LifetimeError {
                    message: format!("Cannot move `{}` while borrowed", path),
                    kind: LifetimeErrorKind::CannotMoveFromBorrow,
                    span: Some(span),
                    related_spans: borrows.iter().map(|b| b.span).collect(),
                });
            }
        }

        self.moved.insert(path.to_string());
        Ok(())
    }

    /// Check if a value is still valid (not moved)
    pub fn is_valid(&self, path: &str) -> bool {
        !self.moved.contains(path)
    }

    /// Check if a path has any active mutable borrows
    pub fn has_mutable_borrow(&self, path: &str) -> bool {
        self.borrows
            .get(path)
            .map(|bs| bs.iter().any(|b| b.is_mutable))
            .unwrap_or(false)
    }

    /// Check if a path has any active borrows
    pub fn has_any_borrow(&self, path: &str) -> bool {
        self.borrows
            .get(path)
            .map(|bs| !bs.is_empty())
            .unwrap_or(false)
    }

    /// Clear all borrows (for new scope)
    pub fn clear(&mut self) {
        self.borrows.clear();
        self.moved.clear();
    }

    /// Create a child scope state
    pub fn child(&self) -> Self {
        Self {
            borrows: self.borrows.clone(),
            moved: self.moved.clone(),
            next_borrow_id: self.next_borrow_id,
        }
    }
}

// ============================================================================
// Lifetime Analyzer
// ============================================================================

/// Analyzes lifetimes and borrow checking
pub struct LifetimeAnalyzer {
    /// Lifetime constraints collected during analysis
    constraints: Vec<LifetimeConstraint>,
    /// Lifetime substitutions (reserved for lifetime inference)
    #[allow(dead_code)]
    substitutions: HashMap<LifetimeId, Lifetime>,
    /// Borrow state for current scope
    borrow_state: BorrowState,
    /// Collected errors
    errors: Vec<LifetimeError>,
    /// Lifetime ID counter
    next_lifetime_id: usize,
}

impl LifetimeAnalyzer {
    /// Create a new lifetime analyzer
    pub fn new() -> Self {
        Self {
            constraints: Vec::new(),
            substitutions: HashMap::new(),
            borrow_state: BorrowState::new(),
            errors: Vec::new(),
            next_lifetime_id: 0,
        }
    }

    /// Create a fresh lifetime
    pub fn fresh_lifetime(&mut self) -> LifetimeId {
        let id = LifetimeId(self.next_lifetime_id);
        self.next_lifetime_id += 1;
        id
    }

    /// Add a constraint that `longer` outlives `shorter`
    pub fn add_outlives_constraint(
        &mut self,
        longer: LifetimeId,
        shorter: LifetimeId,
        span: Option<(usize, usize)>,
    ) {
        self.constraints.push(LifetimeConstraint {
            longer,
            shorter,
            span,
        });
    }

    /// Record a borrow
    pub fn borrow(
        &mut self,
        path: String,
        lifetime: LifetimeId,
        is_mutable: bool,
        span: (usize, usize),
    ) -> Option<BorrowId> {
        match self.borrow_state.borrow(path, lifetime, is_mutable, span) {
            Ok(id) => Some(id),
            Err(error) => {
                self.errors.push(error);
                None
            }
        }
    }

    /// End a borrow
    pub fn end_borrow(&mut self, id: BorrowId) {
        self.borrow_state.end_borrow(id);
    }

    /// Record a move
    pub fn move_value(&mut self, path: &str, span: (usize, usize)) {
        if let Err(error) = self.borrow_state.move_value(path, span) {
            self.errors.push(error);
        }
    }

    /// Check a use of a value
    pub fn check_use(&mut self, path: &str, span: (usize, usize)) {
        if !self.borrow_state.is_valid(path) {
            self.errors.push(LifetimeError {
                message: format!("Use of moved value: `{}`", path),
                kind: LifetimeErrorKind::UseAfterMove,
                span: Some(span),
                related_spans: vec![],
            });
        }
    }

    /// Check assignment to a borrowed value
    pub fn check_assignment(&mut self, path: &str, span: (usize, usize)) {
        if self.borrow_state.has_any_borrow(path) {
            self.errors.push(LifetimeError {
                message: format!("Cannot assign to `{}` while borrowed", path),
                kind: LifetimeErrorKind::CannotAssignToImmutable,
                span: Some(span),
                related_spans: vec![],
            });
        }
    }

    /// Infer lifetime for a reference type
    pub fn infer_reference_lifetime(&mut self, ty: &RustType) -> Option<LifetimeId> {
        match ty {
            RustType::Reference { lifetime, .. } => {
                if let Some(Lifetime::Named { id, .. } | Lifetime::Inferred(id)) = lifetime {
                    Some(*id)
                } else if let Some(Lifetime::Static) = lifetime {
                    // 'static has a special ID
                    Some(LifetimeId(usize::MAX))
                } else {
                    // Create a fresh lifetime for anonymous/none
                    Some(self.fresh_lifetime())
                }
            }
            _ => None,
        }
    }

    /// Check if a type contains any lifetime that needs checking
    pub fn type_has_lifetime(&self, ty: &RustType) -> bool {
        match ty {
            RustType::Reference { .. } => true,
            RustType::Named { type_args, .. } => {
                type_args.iter().any(|t| self.type_has_lifetime(t))
            }
            RustType::Tuple(elements) => elements.iter().any(|t| self.type_has_lifetime(t)),
            RustType::Array { element, .. } | RustType::Slice(element) => {
                self.type_has_lifetime(element)
            }
            _ => false,
        }
    }

    /// Validate all collected constraints
    pub fn validate_constraints(&mut self) -> bool {
        // Build a graph of lifetime relationships
        let mut outlives: HashMap<LifetimeId, HashSet<LifetimeId>> = HashMap::new();

        for constraint in &self.constraints {
            outlives
                .entry(constraint.longer)
                .or_default()
                .insert(constraint.shorter);
        }

        // Check for cycles (which would indicate unsatisfiable constraints)
        for constraint in &self.constraints {
            if self.has_cycle(&outlives, constraint.shorter, constraint.longer) {
                self.errors.push(LifetimeError {
                    message: "Lifetime constraint cycle detected".to_string(),
                    kind: LifetimeErrorKind::ConstraintNotSatisfied,
                    span: constraint.span,
                    related_spans: vec![],
                });
                return false;
            }
        }

        true
    }

    /// Check if there's a cycle in lifetime constraints
    fn has_cycle(
        &self,
        outlives: &HashMap<LifetimeId, HashSet<LifetimeId>>,
        from: LifetimeId,
        to: LifetimeId,
    ) -> bool {
        let mut visited = HashSet::new();
        let mut stack = vec![from];

        while let Some(current) = stack.pop() {
            if current == to {
                return true;
            }

            if visited.insert(current) {
                if let Some(children) = outlives.get(&current) {
                    stack.extend(children.iter().copied());
                }
            }
        }

        false
    }

    /// Get collected errors
    pub fn errors(&self) -> &[LifetimeError] {
        &self.errors
    }

    /// Check if analysis has errors
    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    /// Clear errors
    pub fn clear_errors(&mut self) {
        self.errors.clear();
    }

    /// Enter a new scope
    pub fn enter_scope(&mut self) -> BorrowState {
        let child = self.borrow_state.child();
        std::mem::replace(&mut self.borrow_state, child)
    }

    /// Exit a scope, restoring previous state
    pub fn exit_scope(&mut self, previous: BorrowState) {
        self.borrow_state = previous;
    }
}

impl Default for LifetimeAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_borrow_state_basic() {
        let mut state = BorrowState::new();
        let lt = LifetimeId(0);

        // Should succeed: first immutable borrow
        let result = state.borrow("x".to_string(), lt, false, (0, 1));
        assert!(result.is_ok());

        // Should succeed: second immutable borrow
        let result = state.borrow("x".to_string(), lt, false, (2, 3));
        assert!(result.is_ok());
    }

    #[test]
    fn test_borrow_state_mutable_conflict() {
        let mut state = BorrowState::new();
        let lt = LifetimeId(0);

        // First mutable borrow
        let result = state.borrow("x".to_string(), lt, true, (0, 1));
        assert!(result.is_ok());

        // Second borrow should fail (mutable + any)
        let result = state.borrow("x".to_string(), lt, false, (2, 3));
        assert!(result.is_err());
    }

    #[test]
    fn test_borrow_state_immutable_then_mutable() {
        let mut state = BorrowState::new();
        let lt = LifetimeId(0);

        // First immutable borrow
        let result = state.borrow("x".to_string(), lt, false, (0, 1));
        assert!(result.is_ok());

        // Mutable borrow should fail
        let result = state.borrow("x".to_string(), lt, true, (2, 3));
        assert!(result.is_err());
    }

    #[test]
    fn test_borrow_state_move() {
        let mut state = BorrowState::new();

        // Move should succeed
        let result = state.move_value("x", (0, 1));
        assert!(result.is_ok());

        // Value should no longer be valid
        assert!(!state.is_valid("x"));

        // Borrow after move should fail
        let lt = LifetimeId(0);
        let result = state.borrow("x".to_string(), lt, false, (2, 3));
        assert!(result.is_err());
    }

    #[test]
    fn test_borrow_state_move_while_borrowed() {
        let mut state = BorrowState::new();
        let lt = LifetimeId(0);

        // Borrow first
        let result = state.borrow("x".to_string(), lt, false, (0, 1));
        assert!(result.is_ok());

        // Move should fail
        let result = state.move_value("x", (2, 3));
        assert!(result.is_err());
    }

    #[test]
    fn test_lifetime_analyzer_basic() {
        let mut analyzer = LifetimeAnalyzer::new();

        let lt1 = analyzer.fresh_lifetime();
        let lt2 = analyzer.fresh_lifetime();

        // Add constraint: lt1 outlives lt2
        analyzer.add_outlives_constraint(lt1, lt2, None);

        // Should validate successfully
        assert!(analyzer.validate_constraints());
        assert!(!analyzer.has_errors());
    }

    #[test]
    fn test_lifetime_analyzer_use_after_move() {
        let mut analyzer = LifetimeAnalyzer::new();

        // Move value
        analyzer.move_value("x", (0, 1));

        // Use after move should error
        analyzer.check_use("x", (2, 3));

        assert!(analyzer.has_errors());
        assert_eq!(analyzer.errors()[0].kind, LifetimeErrorKind::UseAfterMove);
    }

    #[test]
    fn test_lifetime_analyzer_scope() {
        let mut analyzer = LifetimeAnalyzer::new();
        let lt = analyzer.fresh_lifetime();

        // Borrow in outer scope
        let outer = analyzer.enter_scope();
        let id = analyzer.borrow("x".to_string(), lt, false, (0, 1));
        assert!(id.is_some());

        // Exit scope
        analyzer.exit_scope(outer);

        // In new scope, should be able to mutably borrow
        let id = analyzer.borrow("x".to_string(), lt, true, (2, 3));
        assert!(id.is_some());
    }
}
