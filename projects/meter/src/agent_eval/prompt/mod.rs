// SPEC-MANAGED: projects/meter/tech-design/semantic/source/projects-meter-src-agent-eval-prompt-mod-rs.md#source
// CODEGEN-BEGIN
//! Prompt template system for agent evaluation

pub mod engine;
pub mod registry;
pub mod template;

pub use engine::PromptEngine;
pub use registry::PromptRegistry;
pub use template::{FewShotExample, PromptContext, PromptSection, PromptTemplate, PromptVariable};
// CODEGEN-END
