---
id: projects-meter-src-agent-eval-cost-rs
fill_sections: [overview, source, changes]
capability_refs:
  - id: agent-use-first-cli
    role: primary
    gap: json-default-report-envelope-and-findings
    claim: json-default-report-envelope-and-findings
    coverage: full
    rationale: "Source template implements meter agent-facing CLI, runner, or report surfaces."
---

# Standardized projects/meter/src/agent_eval/cost.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/meter/src/agent_eval/cost.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `CostBreakdown` | projects/meter/src/agent_eval/cost.rs | struct | pub | 234 |  |
| `CostCalculator` | projects/meter/src/agent_eval/cost.rs | struct | pub | 141 |  |
| `ModelPricing` | projects/meter/src/agent_eval/cost.rs | struct | pub | 11 |  |
| `PricingRegistry` | projects/meter/src/agent_eval/cost.rs | struct | pub | 46 |  |
| `add` | projects/meter/src/agent_eval/cost.rs | function | pub | 115 | add(&mut self, pricing: ModelPricing) |
| `calculate_cost` | projects/meter/src/agent_eval/cost.rs | function | pub | 175 | calculate_cost(         &self,         model: &str,         prompt_tokens: u32,         completion_tokens: u32,     ) -> Result<f64, String> |
| `calculate_cost_breakdown` | projects/meter/src/agent_eval/cost.rs | function | pub | 194 | calculate_cost_breakdown(         &self,         model: &str,         prompt_tokens: u32,         completion_tokens: u32,     ) -> Result<CostBreakdown, String> |
| `get` | projects/meter/src/agent_eval/cost.rs | function | pub | 120 | get(&self, model: &str) -> Option<&ModelPricing> |
| `list_models` | projects/meter/src/agent_eval/cost.rs | function | pub | 125 | list_models(&self) -> Vec<String> |
| `list_models_by_provider` | projects/meter/src/agent_eval/cost.rs | function | pub | 130 | list_models_by_provider(&self, provider: &str) -> Vec<String> |
| `new` | projects/meter/src/agent_eval/cost.rs | function | pub | 28 | new(         model: impl Into<String>,         provider: impl Into<String>,         prompt_price: f64,         completion_price: f64,     ) -> Self |
| `new` | projects/meter/src/agent_eval/cost.rs | function | pub | 60 | new() -> Self |
| `new` | projects/meter/src/agent_eval/cost.rs | function | pub | 155 | new() -> Self |
| `registry` | projects/meter/src/agent_eval/cost.rs | function | pub | 221 | registry(&self) -> &PricingRegistry |
| `registry_mut` | projects/meter/src/agent_eval/cost.rs | function | pub | 226 | registry_mut(&mut self) -> &mut PricingRegistry |
| `with_registry` | projects/meter/src/agent_eval/cost.rs | function | pub | 162 | with_registry(registry: PricingRegistry) -> Self |
## Source
<!-- type: rust-source-unit lang: rust -->

````rust
//! Cost calculation system for LLM model pricing

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Model pricing information
#[derive(Debug, Clone, Serialize, Deserialize)]
/// @spec projects/meter/tech-design/semantic/source/projects-meter-src-agent-eval-cost-rs.md#source
pub struct ModelPricing {
    /// Model identifier
    pub model: String,

    /// Provider name (e.g., "openai", "anthropic", "google")
    pub provider: String,

    /// Prompt token price per 1 million tokens (USD)
    pub prompt_price_per_1m: f64,

    /// Completion token price per 1 million tokens (USD)
    pub completion_price_per_1m: f64,
}

/// @spec projects/meter/tech-design/semantic/source/projects-meter-src-agent-eval-cost-rs.md#source
impl ModelPricing {
    /// Create a new model pricing entry
    pub fn new(
        model: impl Into<String>,
        provider: impl Into<String>,
        prompt_price: f64,
        completion_price: f64,
    ) -> Self {
        Self {
            model: model.into(),
            provider: provider.into(),
            prompt_price_per_1m: prompt_price,
            completion_price_per_1m: completion_price,
        }
    }
}

/// Registry of model pricing information
#[derive(Debug, Clone)]
/// @spec projects/meter/tech-design/semantic/source/projects-meter-src-agent-eval-cost-rs.md#source
pub struct PricingRegistry {
    prices: HashMap<String, ModelPricing>,
}

/// @spec projects/meter/tech-design/semantic/source/projects-meter-src-agent-eval-cost-rs.md#source
impl Default for PricingRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// @spec projects/meter/tech-design/semantic/source/projects-meter-src-agent-eval-cost-rs.md#source
impl PricingRegistry {
    /// Create a new pricing registry with pre-populated common models
    pub fn new() -> Self {
        let mut registry = Self {
            prices: HashMap::new(),
        };

        // OpenAI models (as of January 2025)
        registry.add(ModelPricing::new("gpt-4o", "openai", 2.50, 10.00));
        registry.add(ModelPricing::new("gpt-4o-mini", "openai", 0.15, 0.60));
        registry.add(ModelPricing::new("gpt-4-turbo", "openai", 10.00, 30.00));
        registry.add(ModelPricing::new("gpt-4", "openai", 30.00, 60.00));
        registry.add(ModelPricing::new("gpt-3.5-turbo", "openai", 0.50, 1.50));

        // Anthropic Claude models (as of January 2025)
        registry.add(ModelPricing::new(
            "claude-3-opus",
            "anthropic",
            15.00,
            75.00,
        ));
        registry.add(ModelPricing::new(
            "claude-3-opus-20240229",
            "anthropic",
            15.00,
            75.00,
        ));
        registry.add(ModelPricing::new(
            "claude-3-sonnet",
            "anthropic",
            3.00,
            15.00,
        ));
        registry.add(ModelPricing::new(
            "claude-3-sonnet-20240229",
            "anthropic",
            3.00,
            15.00,
        ));
        registry.add(ModelPricing::new("claude-3-haiku", "anthropic", 0.25, 1.25));
        registry.add(ModelPricing::new(
            "claude-3-haiku-20240307",
            "anthropic",
            0.25,
            1.25,
        ));

        // Google Gemini models (as of January 2025)
        registry.add(ModelPricing::new("gemini-pro", "google", 0.50, 1.50));
        registry.add(ModelPricing::new("gemini-pro-vision", "google", 0.50, 1.50));
        registry.add(ModelPricing::new("gemini-1.5-pro", "google", 3.50, 10.50));
        registry.add(ModelPricing::new("gemini-1.5-flash", "google", 0.35, 1.05));

        registry
    }

    /// Add or update a model pricing
    pub fn add(&mut self, pricing: ModelPricing) {
        self.prices.insert(pricing.model.clone(), pricing);
    }

    /// Get pricing for a model
    pub fn get(&self, model: &str) -> Option<&ModelPricing> {
        self.prices.get(model)
    }

    /// List all models
    pub fn list_models(&self) -> Vec<String> {
        self.prices.keys().cloned().collect()
    }

    /// List models by provider
    pub fn list_models_by_provider(&self, provider: &str) -> Vec<String> {
        self.prices
            .values()
            .filter(|p| p.provider == provider)
            .map(|p| p.model.clone())
            .collect()
    }
}

/// Cost calculator
/// @spec projects/meter/tech-design/semantic/source/projects-meter-src-agent-eval-cost-rs.md#source
pub struct CostCalculator {
    registry: PricingRegistry,
}

/// @spec projects/meter/tech-design/semantic/source/projects-meter-src-agent-eval-cost-rs.md#source
impl Default for CostCalculator {
    fn default() -> Self {
        Self::new()
    }
}

/// @spec projects/meter/tech-design/semantic/source/projects-meter-src-agent-eval-cost-rs.md#source
impl CostCalculator {
    /// Create a new cost calculator with default pricing registry
    pub fn new() -> Self {
        Self {
            registry: PricingRegistry::new(),
        }
    }

    /// Create a cost calculator with custom pricing registry
    pub fn with_registry(registry: PricingRegistry) -> Self {
        Self { registry }
    }

    /// Calculate cost for a given model and token usage
    ///
    /// # Arguments
    /// * `model` - Model identifier
    /// * `prompt_tokens` - Number of prompt tokens
    /// * `completion_tokens` - Number of completion tokens
    ///
    /// # Returns
    /// Total cost in USD, or error if model pricing not found
    pub fn calculate_cost(
        &self,
        model: &str,
        prompt_tokens: u32,
        completion_tokens: u32,
    ) -> Result<f64, String> {
        let pricing = self
            .registry
            .get(model)
            .ok_or_else(|| format!("Model pricing not found for: {}", model))?;

        let prompt_cost = (prompt_tokens as f64 / 1_000_000.0) * pricing.prompt_price_per_1m;
        let completion_cost =
            (completion_tokens as f64 / 1_000_000.0) * pricing.completion_price_per_1m;

        Ok(prompt_cost + completion_cost)
    }

    /// Calculate cost breakdown
    pub fn calculate_cost_breakdown(
        &self,
        model: &str,
        prompt_tokens: u32,
        completion_tokens: u32,
    ) -> Result<CostBreakdown, String> {
        let pricing = self
            .registry
            .get(model)
            .ok_or_else(|| format!("Model pricing not found for: {}", model))?;

        let prompt_cost = (prompt_tokens as f64 / 1_000_000.0) * pricing.prompt_price_per_1m;
        let completion_cost =
            (completion_tokens as f64 / 1_000_000.0) * pricing.completion_price_per_1m;

        Ok(CostBreakdown {
            model: model.to_string(),
            prompt_tokens,
            completion_tokens,
            total_tokens: prompt_tokens + completion_tokens,
            prompt_cost_usd: prompt_cost,
            completion_cost_usd: completion_cost,
            total_cost_usd: prompt_cost + completion_cost,
        })
    }

    /// Get the underlying pricing registry
    pub fn registry(&self) -> &PricingRegistry {
        &self.registry
    }

    /// Get mutable reference to pricing registry (for custom pricing)
    pub fn registry_mut(&mut self) -> &mut PricingRegistry {
        &mut self.registry
    }
}

/// Detailed cost breakdown
#[derive(Debug, Clone, Serialize, Deserialize)]
/// @spec projects/meter/tech-design/semantic/source/projects-meter-src-agent-eval-cost-rs.md#source
pub struct CostBreakdown {
    /// Model used
    pub model: String,

    /// Prompt tokens
    pub prompt_tokens: u32,

    /// Completion tokens
    pub completion_tokens: u32,

    /// Total tokens
    pub total_tokens: u32,

    /// Prompt cost in USD
    pub prompt_cost_usd: f64,

    /// Completion cost in USD
    pub completion_cost_usd: f64,

    /// Total cost in USD
    pub total_cost_usd: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_model_pricing() {
        let pricing = ModelPricing::new("gpt-4o-mini", "openai", 0.15, 0.60);
        assert_eq!(pricing.model, "gpt-4o-mini");
        assert_eq!(pricing.provider, "openai");
        assert_eq!(pricing.prompt_price_per_1m, 0.15);
        assert_eq!(pricing.completion_price_per_1m, 0.60);
    }

    #[test]
    fn test_pricing_registry_default() {
        let registry = PricingRegistry::new();

        // Check OpenAI models
        assert!(registry.get("gpt-4o-mini").is_some());
        assert!(registry.get("gpt-4o").is_some());

        // Check Anthropic models
        assert!(registry.get("claude-3-opus").is_some());
        assert!(registry.get("claude-3-sonnet").is_some());
        assert!(registry.get("claude-3-haiku").is_some());

        // Check Google models
        assert!(registry.get("gemini-1.5-pro").is_some());
        assert!(registry.get("gemini-1.5-flash").is_some());
    }

    #[test]
    fn test_pricing_registry_list_by_provider() {
        let registry = PricingRegistry::new();

        let openai_models = registry.list_models_by_provider("openai");
        assert!(openai_models.contains(&"gpt-4o-mini".to_string()));
        assert!(openai_models.contains(&"gpt-4o".to_string()));

        let anthropic_models = registry.list_models_by_provider("anthropic");
        assert!(anthropic_models.contains(&"claude-3-opus".to_string()));
        assert!(anthropic_models.contains(&"claude-3-sonnet".to_string()));
    }

    #[test]
    fn test_cost_calculator_gpt4o_mini() {
        let calculator = CostCalculator::new();

        // gpt-4o-mini: $0.15 per 1M prompt tokens, $0.60 per 1M completion tokens
        // 100 prompt tokens = $0.000015
        // 50 completion tokens = $0.000030
        // Total = $0.000045
        let cost = calculator.calculate_cost("gpt-4o-mini", 100, 50).unwrap();
        assert!((cost - 0.000045).abs() < 1e-9);
    }

    #[test]
    fn test_cost_calculator_claude_opus() {
        let calculator = CostCalculator::new();

        // claude-3-opus: $15.00 per 1M prompt tokens, $75.00 per 1M completion tokens
        // 1000 prompt tokens = $0.015
        // 500 completion tokens = $0.0375
        // Total = $0.0525
        let cost = calculator
            .calculate_cost("claude-3-opus", 1000, 500)
            .unwrap();
        assert!((cost - 0.0525).abs() < 1e-9);
    }

    #[test]
    fn test_cost_calculator_unknown_model() {
        let calculator = CostCalculator::new();
        let result = calculator.calculate_cost("unknown-model", 100, 50);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Model pricing not found"));
    }

    #[test]
    fn test_cost_breakdown() {
        let calculator = CostCalculator::new();
        let breakdown = calculator
            .calculate_cost_breakdown("gpt-4o-mini", 100, 50)
            .unwrap();

        assert_eq!(breakdown.model, "gpt-4o-mini");
        assert_eq!(breakdown.prompt_tokens, 100);
        assert_eq!(breakdown.completion_tokens, 50);
        assert_eq!(breakdown.total_tokens, 150);
        assert!((breakdown.prompt_cost_usd - 0.000015).abs() < 1e-9);
        assert!((breakdown.completion_cost_usd - 0.000030).abs() < 1e-9);
        assert!((breakdown.total_cost_usd - 0.000045).abs() < 1e-9);
    }

    #[test]
    fn test_custom_pricing() {
        let mut registry = PricingRegistry::new();
        registry.add(ModelPricing::new("custom-model", "custom", 1.0, 2.0));

        let calculator = CostCalculator::with_registry(registry);
        let cost = calculator
            .calculate_cost("custom-model", 1_000_000, 1_000_000)
            .unwrap();

        // 1M prompt tokens * $1.0 + 1M completion tokens * $2.0 = $3.0
        assert_eq!(cost, 3.0);
    }

    #[test]
    fn test_zero_tokens() {
        let calculator = CostCalculator::new();
        let cost = calculator.calculate_cost("gpt-4o-mini", 0, 0).unwrap();
        assert_eq!(cost, 0.0);
    }

    #[test]
    fn test_large_token_count() {
        let calculator = CostCalculator::new();
        // 1 million tokens each
        let cost = calculator
            .calculate_cost("gpt-4o-mini", 1_000_000, 1_000_000)
            .unwrap();
        // $0.15 + $0.60 = $0.75
        assert!((cost - 0.75).abs() < 1e-9);
    }

    #[test]
    fn test_cost_calculator_claude_sonnet() {
        let calculator = CostCalculator::new();

        // claude-3-sonnet: $3.00 per 1M prompt tokens, $15.00 per 1M completion tokens
        // 10000 prompt tokens = $0.03
        // 5000 completion tokens = $0.075
        // Total = $0.105
        let cost = calculator
            .calculate_cost("claude-3-sonnet", 10000, 5000)
            .unwrap();
        assert!((cost - 0.105).abs() < 1e-9);
    }

    #[test]
    fn test_cost_calculator_claude_haiku() {
        let calculator = CostCalculator::new();

        // claude-3-haiku: $0.25 per 1M prompt tokens, $1.25 per 1M completion tokens
        // 100000 prompt tokens = $0.025
        // 50000 completion tokens = $0.0625
        // Total = $0.0875
        let cost = calculator
            .calculate_cost("claude-3-haiku", 100000, 50000)
            .unwrap();
        assert!((cost - 0.0875).abs() < 1e-9);
    }

    #[test]
    fn test_cost_calculator_gemini_pro() {
        let calculator = CostCalculator::new();

        // gemini-pro: $0.50 per 1M prompt tokens, $1.50 per 1M completion tokens
        // 10000 prompt tokens = $0.005
        // 5000 completion tokens = $0.0075
        // Total = $0.0125
        let cost = calculator
            .calculate_cost("gemini-pro", 10000, 5000)
            .unwrap();
        assert!((cost - 0.0125).abs() < 1e-9);
    }

    #[test]
    fn test_cost_calculator_gemini_1_5_pro() {
        let calculator = CostCalculator::new();

        // gemini-1.5-pro: $3.50 per 1M prompt tokens, $10.50 per 1M completion tokens
        // 10000 prompt tokens = $0.035
        // 5000 completion tokens = $0.0525
        // Total = $0.0875
        let cost = calculator
            .calculate_cost("gemini-1.5-pro", 10000, 5000)
            .unwrap();
        assert!((cost - 0.0875).abs() < 1e-9);
    }

    #[test]
    fn test_cost_calculator_gemini_flash() {
        let calculator = CostCalculator::new();

        // gemini-1.5-flash: $0.35 per 1M prompt tokens, $1.05 per 1M completion tokens
        // 10000 prompt tokens = $0.0035
        // 5000 completion tokens = $0.00525
        // Total = $0.00875
        let cost = calculator
            .calculate_cost("gemini-1.5-flash", 10000, 5000)
            .unwrap();
        assert!((cost - 0.00875).abs() < 1e-9);
    }

    #[test]
    fn test_all_anthropic_models_exist() {
        let registry = PricingRegistry::new();
        let anthropic_models = registry.list_models_by_provider("anthropic");

        // Verify all Claude models are registered
        assert!(anthropic_models.contains(&"claude-3-opus".to_string()));
        assert!(anthropic_models.contains(&"claude-3-sonnet".to_string()));
        assert!(anthropic_models.contains(&"claude-3-haiku".to_string()));
        assert_eq!(anthropic_models.len(), 6); // 3 base + 3 versioned
    }

    #[test]
    fn test_all_google_models_exist() {
        let registry = PricingRegistry::new();
        let google_models = registry.list_models_by_provider("google");

        // Verify all Gemini models are registered
        assert!(google_models.contains(&"gemini-pro".to_string()));
        assert!(google_models.contains(&"gemini-1.5-pro".to_string()));
        assert!(google_models.contains(&"gemini-1.5-flash".to_string()));
        assert!(google_models.len() >= 4); // gemini-pro, gemini-pro-vision, gemini-1.5-pro, gemini-1.5-flash
    }

    #[test]
    fn test_claude_versioned_models() {
        let calculator = CostCalculator::new();

        // Test versioned model names
        let cost_opus = calculator
            .calculate_cost("claude-3-opus-20240229", 1000, 500)
            .unwrap();
        let cost_sonnet = calculator
            .calculate_cost("claude-3-sonnet-20240229", 1000, 500)
            .unwrap();
        let cost_haiku = calculator
            .calculate_cost("claude-3-haiku-20240307", 1000, 500)
            .unwrap();

        // Verify costs are calculated correctly
        assert!(cost_opus > 0.0);
        assert!(cost_sonnet > 0.0);
        assert!(cost_haiku > 0.0);

        // Opus should be most expensive, Haiku cheapest
        assert!(cost_opus > cost_sonnet);
        assert!(cost_sonnet > cost_haiku);
    }
}
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/meter/src/agent_eval/cost.rs
    action: modify
    section: rust-source-unit
    impl_mode: codegen
    description: |
      Source template for `projects/meter/src/agent_eval/cost.rs` captured during meter full-codegen standardization.
```
