//! Protocols module — domain contracts for all 5 SDD domains.
//!
//! Protocol types are pure domain contracts: no ORM, no persistence logic.
//! Agents read and write protocol types; consumers (Conductor, etc.) map
//! their storage models to/from protocols.
//!
//! # Protocols
//!
//! | Protocol | Key Fields | Agent |
//! |----------|-----------|-------|
//! | [`ProjectProtocol`] | id, name, repo_url, platform | RestructureCodebaseAgent |
//! | [`IssueProtocol`] | id, title, description, status, priority, labels, acceptance_criteria | RestructureIssueAgent |
//! | [`SpecProtocol`] | id, path, content, format, version, sync_target | ChangeSpecAgent, CodebaseToSpecAgent |
//! | [`ChangeProtocol`] | id, project_id, issue_ids, spec_ids, branch, status | CodeAgent |
//! | [`CodeIndexProtocol`] | module_path, endpoints, models, dependencies | ReferenceCodebaseContextAgent |

mod change;
mod code_index;
mod issue;
mod project;
mod spec;

pub use change::{ChangeProtocol, ChangeStatus};
pub use code_index::CodeIndexProtocol;
pub use issue::{IssuePriority, IssueProtocol, IssueStatus};
pub use project::{Platform, ProjectProtocol};
pub use spec::{SpecFormat, SpecProtocol};
