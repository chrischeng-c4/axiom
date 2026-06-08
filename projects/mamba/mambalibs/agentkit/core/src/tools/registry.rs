use crate::error::{NovaError, NovaResult};
use crate::tools::tool::Tool;
use dashmap::DashMap;
use std::sync::Arc;

/// Global tool registry (thread-safe)
pub struct ToolRegistry {
    tools: DashMap<String, Arc<dyn Tool>>,
}

impl ToolRegistry {
    pub fn new() -> Self {
        Self {
            tools: DashMap::new(),
        }
    }

    pub fn register(&self, tool: Arc<dyn Tool>) -> NovaResult<()> {
        let name = tool.name().to_string();

        if self.tools.contains_key(&name) {
            return Err(NovaError::ValidationFailed(format!(
                "Tool '{}' is already registered",
                name
            )));
        }

        self.tools.insert(name, tool);
        Ok(())
    }

    pub fn unregister(&self, name: &str) -> NovaResult<()> {
        self.tools
            .remove(name)
            .ok_or_else(|| NovaError::ToolNotFound(format!("Tool '{}' not found", name)))?;
        Ok(())
    }

    pub fn get(&self, name: &str) -> Option<Arc<dyn Tool>> {
        self.tools.get(name).map(|entry| entry.value().clone())
    }

    pub fn contains(&self, name: &str) -> bool {
        self.tools.contains_key(name)
    }

    pub fn tool_names(&self) -> Vec<String> {
        self.tools.iter().map(|entry| entry.key().clone()).collect()
    }

    pub fn count(&self) -> usize {
        self.tools.len()
    }

    pub fn clear(&self) {
        self.tools.clear();
    }
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tools::tool::ToolParameter;
    use async_trait::async_trait;

    struct TestTool;

    #[async_trait]
    impl Tool for TestTool {
        fn name(&self) -> &str {
            "test"
        }

        fn description(&self) -> &str {
            "Test tool"
        }

        fn parameters(&self) -> Vec<ToolParameter> {
            vec![]
        }

        async fn execute(
            &self,
            _arguments: serde_json::Value,
        ) -> crate::error::NovaResult<serde_json::Value> {
            Ok(serde_json::json!({}))
        }
    }

    #[test]
    fn test_registry() {
        let registry = ToolRegistry::new();

        assert_eq!(registry.count(), 0);

        registry.register(Arc::new(TestTool)).unwrap();
        assert_eq!(registry.count(), 1);
        assert!(registry.contains("test"));

        let retrieved = registry.get("test").unwrap();
        assert_eq!(retrieved.name(), "test");

        registry.unregister("test").unwrap();
        assert_eq!(registry.count(), 0);
    }
}
