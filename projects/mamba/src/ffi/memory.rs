/// Memory lifecycle bridge for FFI (#268).
///
/// Tracks heap allocations crossing the FFI boundary and generates
/// corresponding free/cleanup calls to prevent leaks.
use super::c_types::CType;

/// An allocation that crosses the FFI boundary and needs cleanup.
#[derive(Debug, Clone)]
pub struct FfiAllocation {
    /// Variable name holding the pointer
    pub var_name: String,
    /// C type of the allocation
    pub c_type: CType,
    /// Name of the free function (e.g., "free", "mamba_free_string")
    pub free_fn: String,
}

/// Memory bridge tracks allocations needing cleanup in a scope (#268).
#[derive(Debug, Clone, Default)]
pub struct MemoryBridge {
    allocations: Vec<FfiAllocation>,
}

impl MemoryBridge {
    pub fn new() -> Self {
        Self::default()
    }

    /// Track a new allocation that will need cleanup.
    pub fn track(&mut self, alloc: FfiAllocation) {
        self.allocations.push(alloc);
    }

    /// Get all tracked allocations.
    pub fn allocations(&self) -> &[FfiAllocation] {
        &self.allocations
    }

    /// Generate cleanup code (Rust source) for all tracked allocations (#268).
    pub fn generate_cleanup(&self) -> String {
        let mut out = String::new();
        for alloc in &self.allocations {
            out.push_str(&format!(
                "    if !{var}.is_null() {{ {free}({var}); }}\n",
                var = alloc.var_name,
                free = alloc.free_fn,
            ));
        }
        out
    }

    /// Clear all tracked allocations (after cleanup is emitted).
    pub fn clear(&mut self) {
        self.allocations.clear();
    }
}

/// Determine the appropriate free function for a C type (#268).
pub fn free_fn_for_type(c_type: &CType) -> &'static str {
    match c_type {
        CType::ConstChar | CType::MutChar => "mamba_free_string",
        CType::Pointer(_) | CType::ConstPointer(_) => "free",
        _ => "free",
    }
}

/// Generate Rust source for the string memory management helpers (#268).
pub fn generate_string_helpers() -> &'static str {
    "/// Convert a Rust string to a C string (caller must free with mamba_free_string).\n\
     #[no_mangle]\n\
     pub extern \"C\" fn mamba_string_to_c(s: &str) -> *mut std::ffi::c_char {\n\
     \x20   match std::ffi::CString::new(s) {\n\
     \x20       Ok(cs) => cs.into_raw(),\n\
     \x20       Err(_) => std::ptr::null_mut(),\n\
     \x20   }\n\
     }\n\n\
     /// Free a C string allocated by mamba_string_to_c.\n\
     #[no_mangle]\n\
     pub unsafe extern \"C\" fn mamba_free_string(s: *mut std::ffi::c_char) {\n\
     \x20   if !s.is_null() {\n\
     \x20       drop(std::ffi::CString::from_raw(s));\n\
     \x20   }\n\
     }\n\n\
     /// Convert a C string to a Rust &str (borrowed, no allocation).\n\
     #[no_mangle]\n\
     pub unsafe extern \"C\" fn mamba_c_to_str<'a>(s: *const std::ffi::c_char) -> &'a str {\n\
     \x20   if s.is_null() {\n\
     \x20       return \"\";\n\
     \x20   }\n\
     \x20   std::ffi::CStr::from_ptr(s).to_str().unwrap_or(\"\")\n\
     }\n"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_bridge_track_and_cleanup() {
        let mut bridge = MemoryBridge::new();
        bridge.track(FfiAllocation {
            var_name: "ptr".into(),
            c_type: CType::Pointer(Box::new(CType::Int32)),
            free_fn: "free".into(),
        });
        bridge.track(FfiAllocation {
            var_name: "name".into(),
            c_type: CType::MutChar,
            free_fn: "mamba_free_string".into(),
        });

        let cleanup = bridge.generate_cleanup();
        assert!(cleanup.contains("free(ptr)"));
        assert!(cleanup.contains("mamba_free_string(name)"));
        assert_eq!(bridge.allocations().len(), 2);

        bridge.clear();
        assert!(bridge.allocations().is_empty());
    }

    #[test]
    fn test_free_fn_for_type() {
        assert_eq!(free_fn_for_type(&CType::ConstChar), "mamba_free_string");
        assert_eq!(free_fn_for_type(&CType::MutChar), "mamba_free_string");
        assert_eq!(
            free_fn_for_type(&CType::Pointer(Box::new(CType::Int32))),
            "free"
        );
    }

    #[test]
    fn test_string_helpers() {
        let helpers = generate_string_helpers();
        assert!(helpers.contains("mamba_string_to_c"));
        assert!(helpers.contains("mamba_free_string"));
        assert!(helpers.contains("mamba_c_to_str"));
        assert!(helpers.contains("CString::new"));
        assert!(helpers.contains("CString::from_raw"));
    }
}
