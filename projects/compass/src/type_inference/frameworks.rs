//! Framework support (Sprint 6 - Track 1)
//!
//! Provides type inference for popular Python frameworks:
//! - Django: models, views, templates
//! - Flask: routes, blueprints
//! - FastAPI: endpoints, dependency injection
//! - Pydantic: models, validators
//! - SQLAlchemy: ORM mappings

use std::collections::HashMap;
use std::path::PathBuf;

use super::deep_inference::TypeContext;
use super::package_managers::{PackageManagerDetection, PackageManagerDetector};
use crate::type_inference::ty::Type;

// ============================================================================
// Framework Detection
// ============================================================================

/// Detected framework in a project.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Framework {
    Django,
    Flask,
    FastAPI,
    Pydantic,
    SQLAlchemy,
    Celery,
    Custom(String),
}

/// Framework detection result.
#[derive(Debug, Clone)]
pub struct FrameworkDetection {
    /// Detected frameworks
    pub frameworks: Vec<Framework>,
    /// Framework-specific files
    pub framework_files: HashMap<Framework, Vec<PathBuf>>,
    /// Confidence scores (0.0 to 1.0)
    pub confidence: HashMap<Framework, f64>,
}

impl FrameworkDetection {
    /// Create empty detection.
    pub fn empty() -> Self {
        Self {
            frameworks: Vec::new(),
            framework_files: HashMap::new(),
            confidence: HashMap::new(),
        }
    }

    /// Check if a framework was detected.
    pub fn has_framework(&self, framework: &Framework) -> bool {
        self.frameworks.contains(framework)
    }

    /// Get confidence for a framework.
    pub fn confidence_for(&self, framework: &Framework) -> f64 {
        self.confidence.get(framework).copied().unwrap_or(0.0)
    }

    /// Add a detected framework with confidence.
    ///
    /// If the framework was already detected, takes the maximum confidence.
    pub fn add_framework(&mut self, framework: Framework, confidence: f64) {
        if !self.frameworks.contains(&framework) {
            self.frameworks.push(framework.clone());
        }

        // Take maximum of existing and new confidence
        let current_confidence = self.confidence.get(&framework).copied().unwrap_or(0.0);
        let max_confidence = current_confidence.max(confidence);
        self.confidence.insert(framework, max_confidence);
    }
}

/// Detect frameworks in a project.
pub struct FrameworkDetector {
    /// Project root
    root: PathBuf,
}

impl FrameworkDetector {
    /// Create a new detector.
    pub fn new(root: PathBuf) -> Self {
        Self { root }
    }

    /// Detect frameworks in the project.
    ///
    /// Uses multiple detection strategies:
    /// 1. Package manager dependencies (HIGH CONFIDENCE - from lockfiles)
    /// 2. File-based detection (project structure, config files)
    pub fn detect(&self) -> FrameworkDetection {
        let mut result = FrameworkDetection::empty();

        // NEW: Detect via package manager (highest confidence when lockfile exists)
        let pkg_detector = PackageManagerDetector::new(self.root.clone());
        let pkg_detection = pkg_detector.detect();

        // Add frameworks detected from dependencies
        self.detect_from_dependencies(&pkg_detection, &mut result);

        // Continue with file-based detection (may increase confidence)
        self.detect_django(&mut result);
        self.detect_flask(&mut result);
        self.detect_fastapi(&mut result);

        result
    }

    /// Detect frameworks from package manager dependencies
    ///
    /// This provides high-confidence detection when a lockfile is present.
    fn detect_from_dependencies(
        &self,
        pkg_detection: &PackageManagerDetection,
        result: &mut FrameworkDetection,
    ) {
        // Base confidence: higher with lockfile, lower without
        let base_confidence = if pkg_detection.lock_file.is_some() {
            0.95 // Very high confidence with lockfile
        } else {
            0.85 // Still high confidence from explicit dependencies
        };

        // Check each framework dependency
        for dep in &pkg_detection.dependencies {
            match dep.name.as_str() {
                "django" => {
                    result.add_framework(Framework::Django, base_confidence);
                }
                "fastapi" => {
                    result.add_framework(Framework::FastAPI, base_confidence);
                }
                "flask" => {
                    result.add_framework(Framework::Flask, base_confidence);
                }
                "pydantic" => {
                    // Pydantic alone doesn't mean the project uses it as a framework
                    // Only add if no other framework detected
                    if !result.has_framework(&Framework::Django)
                        && !result.has_framework(&Framework::FastAPI)
                    {
                        result.add_framework(Framework::Pydantic, base_confidence * 0.7);
                    }
                }
                "sqlalchemy" => {
                    result.add_framework(Framework::SQLAlchemy, base_confidence);
                }
                "celery" => {
                    result.add_framework(Framework::Celery, base_confidence);
                }
                _ => {}
            }
        }
    }

    /// Check for Django.
    fn detect_django(&self, result: &mut FrameworkDetection) {
        let mut confidence: f64 = 0.0;
        let mut indicators = 0;

        // 1. Check for manage.py (strong indicator)
        let manage_py = self.root.join("manage.py");
        if manage_py.exists() {
            confidence += 0.4;
            indicators += 1;
        }

        // 2. Check for settings.py or settings module
        if self.find_files_recursive("settings.py", 2) {
            confidence += 0.3;
            indicators += 1;
        }

        // 3. Check for models.py files
        if self.find_files_recursive("models.py", 2) {
            confidence += 0.15;
            indicators += 1;
        }

        // 4. Check requirements files for Django (no version check needed)
        if self.check_requirements_for("django", 0.0) {
            confidence += 0.25;
            indicators += 1;
        }

        // 5. Check pyproject.toml for Django
        if self.check_pyproject_for("django") {
            confidence += 0.2;
            indicators += 1;
        }

        if indicators > 0 {
            result.add_framework(Framework::Django, confidence.min(1.0));
        }
    }

    /// Check for Flask.
    fn detect_flask(&self, result: &mut FrameworkDetection) {
        let mut confidence: f64 = 0.0;
        let mut indicators = 0;

        // 1. Check requirements for Flask (no version check needed)
        if self.check_requirements_for("flask", 0.0) {
            confidence += 0.4;
            indicators += 1;
        }

        // 2. Check pyproject.toml
        if self.check_pyproject_for("flask") {
            confidence += 0.3;
            indicators += 1;
        }

        // 3. Check for app.py or similar Flask app files with Flask code
        let app_files = ["app.py", "application.py", "wsgi.py"];
        for app_file in &app_files {
            let path = self.root.join(app_file);
            if path.exists() {
                // Check if file contains Flask code
                if let Ok(content) = std::fs::read_to_string(&path) {
                    if content.contains("from flask")
                        || content.contains("import flask")
                        || content.contains("Flask(__name__)")
                    {
                        confidence += 0.2;
                        indicators += 1;
                        break;
                    }
                }
            }
        }

        // 4. Look for blueprints directory
        if self.root.join("blueprints").is_dir() || self.find_files_recursive("blueprints.py", 2) {
            confidence += 0.1;
            indicators += 1;
        }

        if indicators > 0 {
            result.add_framework(Framework::Flask, confidence.min(1.0));
        }
    }

    /// Check for FastAPI.
    fn detect_fastapi(&self, result: &mut FrameworkDetection) {
        let mut confidence: f64 = 0.0;
        let mut indicators = 0;

        // 1. Check requirements for FastAPI (no version check needed)
        if self.check_requirements_for("fastapi", 0.0) {
            confidence += 0.5;
            indicators += 1;
        }

        // 2. Check pyproject.toml
        if self.check_pyproject_for("fastapi") {
            confidence += 0.3;
            indicators += 1;
        }

        // 3. Check for main.py or app.py with FastAPI code
        let app_files = ["main.py", "app.py", "api.py"];
        for app_file in &app_files {
            let path = self.root.join(app_file);
            if path.exists() {
                // Check if file contains FastAPI code
                if let Ok(content) = std::fs::read_to_string(&path) {
                    if content.contains("from fastapi")
                        || content.contains("import fastapi")
                        || content.contains("FastAPI()")
                    {
                        confidence += 0.15;
                        indicators += 1;
                        break;
                    }
                }
            }
        }

        // 4. Check for routers directory
        if self.root.join("routers").is_dir() || self.root.join("api").is_dir() {
            confidence += 0.05;
            indicators += 1;
        }

        if indicators > 0 {
            result.add_framework(Framework::FastAPI, confidence.min(1.0));
        }
    }

    // Helper methods

    /// Find files recursively with given name.
    fn find_files_recursive(&self, filename: &str, max_depth: usize) -> bool {
        self.find_files_recursive_impl(&self.root, filename, max_depth, 0)
    }

    fn find_files_recursive_impl(
        &self,
        dir: &PathBuf,
        filename: &str,
        max_depth: usize,
        current_depth: usize,
    ) -> bool {
        if current_depth > max_depth {
            return false;
        }

        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();

                // Check if this is the file we're looking for
                if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
                    if file_name == filename {
                        return true;
                    }
                }

                // If it's a directory (not hidden), recurse into it
                if path.is_dir() {
                    if let Some(dir_name) = path.file_name().and_then(|n| n.to_str()) {
                        if !dir_name.starts_with('.') {
                            if self.find_files_recursive_impl(
                                &path,
                                filename,
                                max_depth,
                                current_depth + 1,
                            ) {
                                return true;
                            }
                        }
                    }
                }
            }
        }
        false
    }

    /// Check requirements files for a package.
    fn check_requirements_for(&self, package: &str, min_version: f64) -> bool {
        let req_files = [
            "requirements.txt",
            "requirements/base.txt",
            "requirements/production.txt",
            "dev-requirements.txt",
        ];

        for req_file in &req_files {
            let path = self.root.join(req_file);
            if path.exists() {
                if let Ok(content) = std::fs::read_to_string(&path) {
                    for line in content.lines() {
                        let line = line.trim();
                        if line.to_lowercase().starts_with(package) {
                            // Found the package, check version if needed
                            if min_version > 0.0 {
                                // Simple version check (could be enhanced)
                                if let Some(version_part) = line.split(">=").nth(1) {
                                    if let Some(version_str) =
                                        version_part.split(&['<', '=', ','][..]).next()
                                    {
                                        if let Ok(version) = version_str.trim().parse::<f64>() {
                                            return version >= min_version;
                                        }
                                    }
                                }
                            }
                            return true;
                        }
                    }
                }
            }
        }
        false
    }

    /// Check pyproject.toml for a dependency.
    fn check_pyproject_for(&self, package: &str) -> bool {
        let pyproject = self.root.join("pyproject.toml");
        if pyproject.exists() {
            if let Ok(content) = std::fs::read_to_string(&pyproject) {
                // Simple check - could be enhanced with proper TOML parsing
                return content.to_lowercase().contains(&format!("\"{}\"", package))
                    || content.to_lowercase().contains(&format!("'{}'", package))
                    || content.to_lowercase().contains(&format!("{} = ", package));
            }
        }
        false
    }
}

// ============================================================================
// Framework-Specific Type Providers
// ============================================================================

/// Provides framework-specific type information.
pub trait FrameworkTypeProvider {
    /// Get types for a symbol.
    fn get_type(&self, symbol: &str, context: &TypeContext) -> Option<Type>;

    /// Get attribute types for an object.
    fn get_attribute_type(&self, base_type: &Type, attr: &str) -> Option<Type>;

    /// Get method signatures.
    fn get_method_signature(&self, base_type: &Type, method: &str) -> Option<MethodType>;

    /// Framework name.
    fn framework_name(&self) -> &str;
}

/// Method type with parameters and return.
#[derive(Debug, Clone)]
pub struct MethodType {
    /// Parameter types
    pub params: Vec<(String, Type)>,
    /// Return type
    pub return_type: Type,
    /// Is async
    pub is_async: bool,
}

// ============================================================================
// Django Support
// ============================================================================

/// Django type provider.
pub struct DjangoTypeProvider {
    /// Model definitions
    models: HashMap<String, DjangoModel>,
}

/// Django model definition.
#[derive(Debug, Clone)]
pub struct DjangoModel {
    /// Model name
    pub name: String,
    /// Fields
    pub fields: HashMap<String, DjangoField>,
    /// Related models
    pub relations: Vec<DjangoRelation>,
}

/// Django field.
#[derive(Debug, Clone)]
pub struct DjangoField {
    /// Field name
    pub name: String,
    /// Field type
    pub field_type: DjangoFieldType,
    /// Is nullable
    pub null: bool,
    /// Has default
    pub has_default: bool,
}

/// Django field type.
#[derive(Debug, Clone)]
pub enum DjangoFieldType {
    CharField,
    TextField,
    IntegerField,
    FloatField,
    BooleanField,
    DateField,
    DateTimeField,
    ForeignKey(String),
    OneToOneField(String),
    ManyToManyField(String),
    Custom(String),
}

/// Django relation.
#[derive(Debug, Clone)]
pub struct DjangoRelation {
    /// Relation name
    pub name: String,
    /// Related model
    pub related_model: String,
    /// Relation type
    pub relation_type: DjangoRelationType,
}

/// Django relation type.
#[derive(Debug, Clone)]
pub enum DjangoRelationType {
    ForeignKey,
    OneToOne,
    ManyToMany,
}

impl DjangoTypeProvider {
    /// Create a new provider.
    pub fn new() -> Self {
        Self {
            models: HashMap::new(),
        }
    }

    /// Register a model.
    pub fn register_model(&mut self, model: DjangoModel) {
        self.models.insert(model.name.clone(), model);
    }

    /// Get model info.
    pub fn get_model(&self, name: &str) -> Option<&DjangoModel> {
        self.models.get(name)
    }
}

impl Default for DjangoTypeProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl FrameworkTypeProvider for DjangoTypeProvider {
    fn get_type(&self, symbol: &str, _context: &TypeContext) -> Option<Type> {
        // Check if symbol is a model
        if self.models.contains_key(symbol) {
            Some(Type::Instance {
                name: symbol.to_string(),
                module: Some("models".to_string()),
                type_args: Vec::new(),
            })
        } else {
            None
        }
    }

    fn get_attribute_type(&self, base_type: &Type, attr: &str) -> Option<Type> {
        if let Type::Instance { name, .. } = base_type {
            if let Some(model) = self.models.get(name) {
                if let Some(field) = model.fields.get(attr) {
                    return Some(self.field_type_to_type(&field.field_type));
                }
            }
        }
        None
    }

    fn get_method_signature(&self, base_type: &Type, method: &str) -> Option<MethodType> {
        if let Type::Instance { name, .. } = base_type {
            // Check if it's a model instance or QuerySet
            let is_queryset = name.ends_with("QuerySet");
            let model_name = if is_queryset {
                // Extract model name from "UserQuerySet" -> "User"
                name.strip_suffix("QuerySet").unwrap_or(name)
            } else {
                name.as_str()
            };

            // Check if this is a known model
            if self.models.contains_key(model_name) || is_queryset {
                return match method {
                    // QuerySet methods that return QuerySet
                    "filter" | "exclude" | "select_related" | "prefetch_related" | "annotate"
                    | "order_by" | "distinct" | "defer" | "only" | "using"
                    | "select_for_update" => Some(MethodType {
                        params: vec![], // **kwargs not modeled yet
                        return_type: Type::Instance {
                            name: format!("{}QuerySet", model_name),
                            module: None,
                            type_args: Vec::new(),
                        },
                        is_async: false,
                    }),

                    // get() returns model instance
                    "get" => Some(MethodType {
                        params: vec![],
                        return_type: Type::Instance {
                            name: model_name.to_string(),
                            module: None,
                            type_args: Vec::new(),
                        },
                        is_async: false,
                    }),

                    // all() returns QuerySet
                    "all" => Some(MethodType {
                        params: vec![],
                        return_type: Type::Instance {
                            name: format!("{}QuerySet", model_name),
                            module: None,
                            type_args: Vec::new(),
                        },
                        is_async: false,
                    }),

                    // first() / last() return Optional[Model]
                    "first" | "last" => Some(MethodType {
                        params: vec![],
                        return_type: Type::Optional(Box::new(Type::Instance {
                            name: model_name.to_string(),
                            module: None,
                            type_args: Vec::new(),
                        })),
                        is_async: false,
                    }),

                    // create() returns model instance
                    "create" => Some(MethodType {
                        params: vec![],
                        return_type: Type::Instance {
                            name: model_name.to_string(),
                            module: None,
                            type_args: Vec::new(),
                        },
                        is_async: false,
                    }),

                    // get_or_create() returns tuple (Model, bool)
                    "get_or_create" => Some(MethodType {
                        params: vec![],
                        return_type: Type::Tuple(vec![
                            Type::Instance {
                                name: model_name.to_string(),
                                module: None,
                                type_args: Vec::new(),
                            },
                            Type::Bool,
                        ]),
                        is_async: false,
                    }),

                    // update_or_create() returns tuple (Model, bool)
                    "update_or_create" => Some(MethodType {
                        params: vec![],
                        return_type: Type::Tuple(vec![
                            Type::Instance {
                                name: model_name.to_string(),
                                module: None,
                                type_args: Vec::new(),
                            },
                            Type::Bool,
                        ]),
                        is_async: false,
                    }),

                    // count() returns int
                    "count" => Some(MethodType {
                        params: vec![],
                        return_type: Type::Int,
                        is_async: false,
                    }),

                    // exists() returns bool
                    "exists" => Some(MethodType {
                        params: vec![],
                        return_type: Type::Bool,
                        is_async: false,
                    }),

                    // delete() returns tuple (int, dict)
                    "delete" => Some(MethodType {
                        params: vec![],
                        return_type: Type::Tuple(vec![
                            Type::Int,
                            Type::Dict(Box::new(Type::Str), Box::new(Type::Int)),
                        ]),
                        is_async: false,
                    }),

                    // update() returns int
                    "update" => Some(MethodType {
                        params: vec![],
                        return_type: Type::Int,
                        is_async: false,
                    }),

                    // values() / values_list() return QuerySet (simplified)
                    "values" | "values_list" => Some(MethodType {
                        params: vec![],
                        return_type: Type::Instance {
                            name: format!("{}QuerySet", model_name),
                            module: None,
                            type_args: Vec::new(),
                        },
                        is_async: false,
                    }),

                    // aggregate() returns dict
                    "aggregate" => Some(MethodType {
                        params: vec![],
                        return_type: Type::Dict(Box::new(Type::Str), Box::new(Type::Any)),
                        is_async: false,
                    }),

                    // Model instance methods
                    "save" if !is_queryset => Some(MethodType {
                        params: vec![],
                        return_type: Type::None,
                        is_async: false,
                    }),

                    "refresh_from_db" if !is_queryset => Some(MethodType {
                        params: vec![],
                        return_type: Type::None,
                        is_async: false,
                    }),

                    _ => None,
                };
            }
        }
        None
    }

    fn framework_name(&self) -> &str {
        "Django"
    }
}

impl DjangoTypeProvider {
    fn field_type_to_type(&self, field_type: &DjangoFieldType) -> Type {
        match field_type {
            DjangoFieldType::CharField | DjangoFieldType::TextField => Type::Str,
            DjangoFieldType::IntegerField => Type::Int,
            DjangoFieldType::FloatField => Type::Float,
            DjangoFieldType::BooleanField => Type::Bool,
            DjangoFieldType::DateField | DjangoFieldType::DateTimeField => Type::Instance {
                name: "datetime".to_string(),
                module: Some("datetime".to_string()),
                type_args: Vec::new(),
            },
            DjangoFieldType::ForeignKey(model) | DjangoFieldType::OneToOneField(model) => {
                Type::Instance {
                    name: model.clone(),
                    module: None,
                    type_args: Vec::new(),
                }
            }
            DjangoFieldType::ManyToManyField(model) => Type::List(Box::new(Type::Instance {
                name: model.clone(),
                module: None,
                type_args: Vec::new(),
            })),
            DjangoFieldType::Custom(name) => Type::Instance {
                name: name.clone(),
                module: None,
                type_args: Vec::new(),
            },
        }
    }

    /// Parse Django models from Python source code.
    /// This is a simplified parser that looks for models.Model subclasses and field definitions.
    pub fn parse_models_from_source(&mut self, source: &str, module_path: &str) {
        let lines: Vec<&str> = source.lines().collect();
        let mut i = 0;

        while i < lines.len() {
            let line = lines[i].trim();

            // Look for class definitions that inherit from models.Model
            if line.starts_with("class ") && line.contains("(") {
                if let Some(model_name) = self.extract_model_name(line) {
                    // Check if it inherits from models.Model
                    if line.contains("models.Model") || line.contains("Model") {
                        // Parse the model body
                        let model = self.parse_model_body(&lines, i + 1, &model_name, module_path);
                        self.register_model(model);
                    }
                }
            }

            i += 1;
        }
    }

    fn extract_model_name(&self, line: &str) -> Option<String> {
        // Extract "User" from "class User(models.Model):"
        if let Some(start) = line.find("class ") {
            let rest = &line[start + 6..];
            if let Some(paren) = rest.find('(') {
                return Some(rest[..paren].trim().to_string());
            }
        }
        None
    }

    fn parse_model_body(
        &self,
        lines: &[&str],
        start_idx: usize,
        model_name: &str,
        _module_path: &str,
    ) -> DjangoModel {
        let mut fields = HashMap::new();
        let mut relations = Vec::new();
        let mut i = start_idx;

        // Determine indentation level of the class body
        let base_indent = lines
            .get(start_idx)
            .and_then(|line| {
                let trimmed = line.trim_start();
                if !trimmed.is_empty() {
                    Some(line.len() - trimmed.len())
                } else {
                    None
                }
            })
            .unwrap_or(4);

        while i < lines.len() {
            let line = lines[i];
            let trimmed = line.trim_start();

            // Stop if we've left the class body (de-dented to class level or less)
            if !trimmed.is_empty() && !trimmed.starts_with('#') {
                let current_indent = line.len() - trimmed.len();
                if current_indent < base_indent {
                    break;
                }
            }

            // Look for field definitions (e.g., "name = models.CharField(...)")
            if trimmed.contains(" = models.") {
                if let Some(field) = self.parse_field_definition(trimmed) {
                    if let DjangoFieldType::ForeignKey(ref related_model) = field.field_type {
                        relations.push(DjangoRelation {
                            name: field.name.clone(),
                            related_model: related_model.clone(),
                            relation_type: DjangoRelationType::ForeignKey,
                        });
                    } else if let DjangoFieldType::OneToOneField(ref related_model) =
                        field.field_type
                    {
                        relations.push(DjangoRelation {
                            name: field.name.clone(),
                            related_model: related_model.clone(),
                            relation_type: DjangoRelationType::OneToOne,
                        });
                    } else if let DjangoFieldType::ManyToManyField(ref related_model) =
                        field.field_type
                    {
                        relations.push(DjangoRelation {
                            name: field.name.clone(),
                            related_model: related_model.clone(),
                            relation_type: DjangoRelationType::ManyToMany,
                        });
                    }
                    fields.insert(field.name.clone(), field);
                }
            }

            i += 1;
        }

        DjangoModel {
            name: model_name.to_string(),
            fields,
            relations,
        }
    }

    fn parse_field_definition(&self, line: &str) -> Option<DjangoField> {
        // Extract field name from "name = models.CharField(...)"
        if let Some(eq_pos) = line.find('=') {
            let field_name = line[..eq_pos].trim().to_string();
            let rest = line[eq_pos + 1..].trim();

            // Determine field type
            let field_type = if rest.contains("CharField") {
                DjangoFieldType::CharField
            } else if rest.contains("TextField") {
                DjangoFieldType::TextField
            } else if rest.contains("IntegerField")
                || rest.contains("AutoField")
                || rest.contains("BigAutoField")
            {
                DjangoFieldType::IntegerField
            } else if rest.contains("FloatField") || rest.contains("DecimalField") {
                DjangoFieldType::FloatField
            } else if rest.contains("BooleanField") {
                DjangoFieldType::BooleanField
            } else if rest.contains("DateTimeField") {
                DjangoFieldType::DateTimeField
            } else if rest.contains("DateField") {
                DjangoFieldType::DateField
            } else if rest.contains("ForeignKey") {
                let related_model = self
                    .extract_related_model(rest)
                    .unwrap_or_else(|| "Unknown".to_string());
                DjangoFieldType::ForeignKey(related_model)
            } else if rest.contains("OneToOneField") {
                let related_model = self
                    .extract_related_model(rest)
                    .unwrap_or_else(|| "Unknown".to_string());
                DjangoFieldType::OneToOneField(related_model)
            } else if rest.contains("ManyToManyField") {
                let related_model = self
                    .extract_related_model(rest)
                    .unwrap_or_else(|| "Unknown".to_string());
                DjangoFieldType::ManyToManyField(related_model)
            } else {
                DjangoFieldType::Custom("Unknown".to_string())
            };

            // Check for null=True
            let null = rest.contains("null=True") || rest.contains("null = True");

            // Check for default value
            let has_default = rest.contains("default=") || rest.contains("default =");

            return Some(DjangoField {
                name: field_name,
                field_type,
                null,
                has_default,
            });
        }

        None
    }

    fn extract_related_model(&self, field_definition: &str) -> Option<String> {
        // Extract "User" from "ForeignKey(User, ...)" or "ForeignKey('User', ...)"
        if let Some(start) = field_definition.find('(') {
            let rest = &field_definition[start + 1..];
            if let Some(end) = rest.find(',').or_else(|| rest.find(')')) {
                let model_ref = rest[..end].trim();
                // Remove quotes if present
                let model_name = model_ref.trim_matches(|c| c == '\'' || c == '"');
                // Handle "self" reference
                if model_name == "self" || model_name == "'self'" {
                    return Some("self".to_string());
                }
                return Some(model_name.to_string());
            }
        }
        None
    }
}

// ============================================================================
// FastAPI Support
// ============================================================================

/// FastAPI type provider.
pub struct FastAPITypeProvider {
    /// Registered endpoints
    endpoints: HashMap<String, FastAPIEndpoint>,
    /// Dependency types
    dependencies: HashMap<String, Type>,
}

/// FastAPI endpoint.
#[derive(Debug, Clone)]
pub struct FastAPIEndpoint {
    /// Path
    pub path: String,
    /// HTTP methods
    pub methods: Vec<String>,
    /// Request body type
    pub request_body: Option<Type>,
    /// Response type
    pub response_type: Type,
    /// Dependencies
    pub dependencies: Vec<String>,
}

impl FastAPITypeProvider {
    /// Create a new provider.
    pub fn new() -> Self {
        Self {
            endpoints: HashMap::new(),
            dependencies: HashMap::new(),
        }
    }

    /// Register an endpoint.
    pub fn register_endpoint(&mut self, name: String, endpoint: FastAPIEndpoint) {
        self.endpoints.insert(name, endpoint);
    }

    /// Register a dependency.
    pub fn register_dependency(&mut self, name: String, ty: Type) {
        self.dependencies.insert(name, ty);
    }
}

impl Default for FastAPITypeProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl FrameworkTypeProvider for FastAPITypeProvider {
    fn get_type(&self, symbol: &str, _context: &TypeContext) -> Option<Type> {
        self.dependencies.get(symbol).cloned()
    }

    fn get_attribute_type(&self, _base_type: &Type, _attr: &str) -> Option<Type> {
        None
    }

    fn get_method_signature(&self, base_type: &Type, method: &str) -> Option<MethodType> {
        // FastAPI parameter injection helpers
        match method {
            // Depends() - dependency injection
            // Usage: user: User = Depends(get_current_user)
            "Depends" => Some(MethodType {
                params: vec![(
                    "dependency".to_string(),
                    Type::Callable {
                        params: vec![],
                        ret: Box::new(Type::Any),
                    },
                )],
                return_type: Type::Any, // Will be inferred from the dependency function
                is_async: false,
            }),

            // Path() - path parameter
            // Usage: user_id: int = Path(..., gt=0)
            "Path" => Some(MethodType {
                params: vec![("default".to_string(), Type::Any)],
                return_type: Type::Any, // Returns the type specified in annotation
                is_async: false,
            }),

            // Query() - query parameter
            // Usage: skip: int = Query(0, ge=0)
            "Query" => Some(MethodType {
                params: vec![("default".to_string(), Type::Any)],
                return_type: Type::Any, // Returns the type specified in annotation
                is_async: false,
            }),

            // Body() - request body
            // Usage: user: UserCreate = Body(...)
            "Body" => Some(MethodType {
                params: vec![("default".to_string(), Type::Any)],
                return_type: Type::Any, // Returns the type specified in annotation
                is_async: false,
            }),

            // Header() - header parameter
            // Usage: token: str = Header(...)
            "Header" => Some(MethodType {
                params: vec![("default".to_string(), Type::Any)],
                return_type: Type::Any, // Returns the type specified in annotation
                is_async: false,
            }),

            // Cookie() - cookie parameter
            // Usage: session: str = Cookie(None)
            "Cookie" => Some(MethodType {
                params: vec![("default".to_string(), Type::Any)],
                return_type: Type::Any, // Returns the type specified in annotation
                is_async: false,
            }),

            // File() - file upload
            // Usage: file: bytes = File(...)
            "File" => Some(MethodType {
                params: vec![("default".to_string(), Type::Any)],
                return_type: Type::Instance {
                    name: "bytes".to_string(),
                    module: None,
                    type_args: Vec::new(),
                },
                is_async: false,
            }),

            // UploadFile() - uploaded file object
            // Usage: file: UploadFile = File(...)
            "UploadFile" => Some(MethodType {
                params: vec![],
                return_type: Type::Instance {
                    name: "UploadFile".to_string(),
                    module: Some("fastapi".to_string()),
                    type_args: Vec::new(),
                },
                is_async: false,
            }),

            // Form() - form data
            // Usage: username: str = Form(...)
            "Form" => Some(MethodType {
                params: vec![("default".to_string(), Type::Any)],
                return_type: Type::Any, // Returns the type specified in annotation
                is_async: false,
            }),

            // Response models
            "JSONResponse" => Some(MethodType {
                params: vec![("content".to_string(), Type::Any)],
                return_type: Type::Instance {
                    name: "JSONResponse".to_string(),
                    module: Some("fastapi.responses".to_string()),
                    type_args: Vec::new(),
                },
                is_async: false,
            }),

            "HTMLResponse" => Some(MethodType {
                params: vec![("content".to_string(), Type::Str)],
                return_type: Type::Instance {
                    name: "HTMLResponse".to_string(),
                    module: Some("fastapi.responses".to_string()),
                    type_args: Vec::new(),
                },
                is_async: false,
            }),

            "PlainTextResponse" => Some(MethodType {
                params: vec![("content".to_string(), Type::Str)],
                return_type: Type::Instance {
                    name: "PlainTextResponse".to_string(),
                    module: Some("fastapi.responses".to_string()),
                    type_args: Vec::new(),
                },
                is_async: false,
            }),

            "RedirectResponse" => Some(MethodType {
                params: vec![("url".to_string(), Type::Str)],
                return_type: Type::Instance {
                    name: "RedirectResponse".to_string(),
                    module: Some("fastapi.responses".to_string()),
                    type_args: Vec::new(),
                },
                is_async: false,
            }),

            "StreamingResponse" => Some(MethodType {
                params: vec![("content".to_string(), Type::Any)],
                return_type: Type::Instance {
                    name: "StreamingResponse".to_string(),
                    module: Some("fastapi.responses".to_string()),
                    type_args: Vec::new(),
                },
                is_async: false,
            }),

            // FastAPI() app instance methods
            "FastAPI" if matches!(base_type, Type::Instance { name, .. } if name == "FastAPI") => {
                Some(MethodType {
                    params: vec![],
                    return_type: Type::Instance {
                        name: "FastAPI".to_string(),
                        module: Some("fastapi".to_string()),
                        type_args: Vec::new(),
                    },
                    is_async: false,
                })
            }

            _ => None,
        }
    }

    fn framework_name(&self) -> &str {
        "FastAPI"
    }
}

// ============================================================================
// Pydantic Support
// ============================================================================

/// Pydantic type provider.
pub struct PydanticTypeProvider {
    /// Registered models
    models: HashMap<String, PydanticModel>,
}

/// Pydantic model.
#[derive(Debug, Clone)]
pub struct PydanticModel {
    /// Model name
    pub name: String,
    /// Fields
    pub fields: HashMap<String, PydanticField>,
    /// Validators
    pub validators: Vec<String>,
    /// Config class
    pub config: Option<PydanticConfig>,
}

/// Pydantic field.
#[derive(Debug, Clone)]
pub struct PydanticField {
    /// Field name
    pub name: String,
    /// Field type
    pub ty: Type,
    /// Default value
    pub default: Option<String>,
    /// Field validators
    pub validators: Vec<String>,
    /// Alias
    pub alias: Option<String>,
}

/// Pydantic config.
#[derive(Debug, Clone)]
pub struct PydanticConfig {
    /// Allow extra fields
    pub extra: PydanticExtra,
    /// Validate assignment
    pub validate_assignment: bool,
    /// Use enum values
    pub use_enum_values: bool,
}

/// Pydantic extra handling.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PydanticExtra {
    Allow,
    Forbid,
    Ignore,
}

impl PydanticTypeProvider {
    /// Create a new provider.
    pub fn new() -> Self {
        Self {
            models: HashMap::new(),
        }
    }

    /// Register a model.
    pub fn register_model(&mut self, model: PydanticModel) {
        self.models.insert(model.name.clone(), model);
    }

    /// Get model info.
    pub fn get_model(&self, name: &str) -> Option<&PydanticModel> {
        self.models.get(name)
    }
}

impl Default for PydanticTypeProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl FrameworkTypeProvider for PydanticTypeProvider {
    fn get_type(&self, symbol: &str, _context: &TypeContext) -> Option<Type> {
        if self.models.contains_key(symbol) {
            Some(Type::Instance {
                name: symbol.to_string(),
                module: None,
                type_args: Vec::new(),
            })
        } else {
            None
        }
    }

    fn get_attribute_type(&self, base_type: &Type, attr: &str) -> Option<Type> {
        if let Type::Instance { name, .. } = base_type {
            if let Some(model) = self.models.get(name) {
                if let Some(field) = model.fields.get(attr) {
                    return Some(field.ty.clone());
                }
            }
        }
        None
    }

    fn get_method_signature(&self, base_type: &Type, method: &str) -> Option<MethodType> {
        if let Type::Instance { name, .. } = base_type {
            // Check if this is a known Pydantic model
            if let Some(_model) = self.models.get(name) {
                return match method {
                    // dict() - convert model to dictionary
                    // Usage: user.dict()
                    "dict" => Some(MethodType {
                        params: vec![],
                        return_type: Type::Dict(Box::new(Type::Str), Box::new(Type::Any)),
                        is_async: false,
                    }),

                    // json() - serialize to JSON string
                    // Usage: user.json()
                    "json" => Some(MethodType {
                        params: vec![],
                        return_type: Type::Str,
                        is_async: false,
                    }),

                    // copy() - create a copy of the model
                    // Usage: new_user = user.copy(update={"name": "New Name"})
                    "copy" => Some(MethodType {
                        params: vec![],
                        return_type: Type::Instance {
                            name: name.clone(),
                            module: None,
                            type_args: Vec::new(),
                        },
                        is_async: false,
                    }),

                    // parse_obj() - parse from dictionary (class method)
                    // Usage: User.parse_obj({"name": "Alice"})
                    "parse_obj" => Some(MethodType {
                        params: vec![(
                            "obj".to_string(),
                            Type::Dict(Box::new(Type::Str), Box::new(Type::Any)),
                        )],
                        return_type: Type::Instance {
                            name: name.clone(),
                            module: None,
                            type_args: Vec::new(),
                        },
                        is_async: false,
                    }),

                    // parse_raw() - parse from JSON string (class method)
                    // Usage: User.parse_raw('{"name": "Alice"}')
                    "parse_raw" => Some(MethodType {
                        params: vec![("b".to_string(), Type::Str)],
                        return_type: Type::Instance {
                            name: name.clone(),
                            module: None,
                            type_args: Vec::new(),
                        },
                        is_async: false,
                    }),

                    // parse_file() - parse from JSON file (class method)
                    // Usage: User.parse_file("user.json")
                    "parse_file" => Some(MethodType {
                        params: vec![("path".to_string(), Type::Str)],
                        return_type: Type::Instance {
                            name: name.clone(),
                            module: None,
                            type_args: Vec::new(),
                        },
                        is_async: false,
                    }),

                    // schema() - get JSON schema (class method)
                    // Usage: User.schema()
                    "schema" => Some(MethodType {
                        params: vec![],
                        return_type: Type::Dict(Box::new(Type::Str), Box::new(Type::Any)),
                        is_async: false,
                    }),

                    // schema_json() - get JSON schema as string (class method)
                    // Usage: User.schema_json()
                    "schema_json" => Some(MethodType {
                        params: vec![],
                        return_type: Type::Str,
                        is_async: false,
                    }),

                    // construct() - construct without validation (class method)
                    // Usage: User.construct(name="Alice")
                    "construct" => Some(MethodType {
                        params: vec![],
                        return_type: Type::Instance {
                            name: name.clone(),
                            module: None,
                            type_args: Vec::new(),
                        },
                        is_async: false,
                    }),

                    // from_orm() - create from ORM model (class method)
                    // Usage: UserResponse.from_orm(db_user)
                    "from_orm" => Some(MethodType {
                        params: vec![("obj".to_string(), Type::Any)],
                        return_type: Type::Instance {
                            name: name.clone(),
                            module: None,
                            type_args: Vec::new(),
                        },
                        is_async: false,
                    }),

                    // validate() - validate data (class method)
                    // Usage: User.validate(data)
                    "validate" => Some(MethodType {
                        params: vec![("value".to_string(), Type::Any)],
                        return_type: Type::Instance {
                            name: name.clone(),
                            module: None,
                            type_args: Vec::new(),
                        },
                        is_async: false,
                    }),

                    // update_forward_refs() - update forward references (class method)
                    // Usage: User.update_forward_refs()
                    "update_forward_refs" => Some(MethodType {
                        params: vec![],
                        return_type: Type::None,
                        is_async: false,
                    }),

                    _ => None,
                };
            }
        }
        None
    }

    fn framework_name(&self) -> &str {
        "Pydantic"
    }
}

// ============================================================================
// Framework Registry
// ============================================================================

/// Registry of framework type providers.
pub struct FrameworkRegistry {
    /// Registered providers
    providers: Vec<Box<dyn FrameworkTypeProvider + Send + Sync>>,
}

impl FrameworkRegistry {
    /// Create a new registry.
    pub fn new() -> Self {
        Self {
            providers: Vec::new(),
        }
    }

    /// Register a provider.
    pub fn register(&mut self, provider: Box<dyn FrameworkTypeProvider + Send + Sync>) {
        self.providers.push(provider);
    }

    /// Get type from any provider.
    pub fn get_type(&self, symbol: &str, context: &TypeContext) -> Option<Type> {
        for provider in &self.providers {
            if let Some(ty) = provider.get_type(symbol, context) {
                return Some(ty);
            }
        }
        None
    }

    /// Get attribute type from any provider.
    pub fn get_attribute_type(&self, base_type: &Type, attr: &str) -> Option<Type> {
        for provider in &self.providers {
            if let Some(ty) = provider.get_attribute_type(base_type, attr) {
                return Some(ty);
            }
        }
        None
    }

    /// Get method signature from any provider.
    pub fn get_method_signature(&self, base_type: &Type, method: &str) -> Option<MethodType> {
        for provider in &self.providers {
            if let Some(sig) = provider.get_method_signature(base_type, method) {
                return Some(sig);
            }
        }
        None
    }
}

impl Default for FrameworkRegistry {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_framework_detection() {
        let detector = FrameworkDetector::new(PathBuf::from("."));
        let result = detector.detect();
        assert!(result.frameworks.is_empty()); // Empty project
    }

    #[test]
    fn test_django_provider() {
        let mut provider = DjangoTypeProvider::new();

        let mut fields = HashMap::new();
        fields.insert(
            "name".to_string(),
            DjangoField {
                name: "name".to_string(),
                field_type: DjangoFieldType::CharField,
                null: false,
                has_default: false,
            },
        );

        provider.register_model(DjangoModel {
            name: "User".to_string(),
            fields,
            relations: Vec::new(),
        });

        assert!(provider.get_model("User").is_some());
    }

    #[test]
    fn test_pydantic_provider() {
        let mut provider = PydanticTypeProvider::new();

        provider.register_model(PydanticModel {
            name: "UserModel".to_string(),
            fields: HashMap::new(),
            validators: Vec::new(),
            config: None,
        });

        assert!(provider.get_model("UserModel").is_some());
    }

    #[test]
    fn test_fastapi_method_signatures() {
        let provider = FastAPITypeProvider::new();

        // Test Depends()
        let depends_sig = provider.get_method_signature(&Type::Any, "Depends");
        assert!(depends_sig.is_some());
        let sig = depends_sig.unwrap();
        assert!(matches!(sig.return_type, Type::Any));

        // Test Path()
        let path_sig = provider.get_method_signature(&Type::Any, "Path");
        assert!(path_sig.is_some());
        let sig = path_sig.unwrap();
        assert!(matches!(sig.return_type, Type::Any));

        // Test Query()
        let query_sig = provider.get_method_signature(&Type::Any, "Query");
        assert!(query_sig.is_some());

        // Test JSONResponse
        let json_response_sig = provider.get_method_signature(&Type::Any, "JSONResponse");
        assert!(json_response_sig.is_some());
        let sig = json_response_sig.unwrap();
        assert!(matches!(sig.return_type, Type::Instance { name, .. } if name == "JSONResponse"));
    }

    #[test]
    fn test_pydantic_method_signatures() {
        let mut provider = PydanticTypeProvider::new();

        // Register a model
        provider.register_model(PydanticModel {
            name: "User".to_string(),
            fields: HashMap::new(),
            validators: Vec::new(),
            config: None,
        });

        let user_type = Type::Instance {
            name: "User".to_string(),
            module: None,
            type_args: Vec::new(),
        };

        // Test dict()
        let dict_sig = provider.get_method_signature(&user_type, "dict");
        assert!(dict_sig.is_some());
        let sig = dict_sig.unwrap();
        assert!(matches!(sig.return_type, Type::Dict(..)));

        // Test json()
        let json_sig = provider.get_method_signature(&user_type, "json");
        assert!(json_sig.is_some());
        let sig = json_sig.unwrap();
        assert!(matches!(sig.return_type, Type::Str));

        // Test copy()
        let copy_sig = provider.get_method_signature(&user_type, "copy");
        assert!(copy_sig.is_some());
        let sig = copy_sig.unwrap();
        assert!(matches!(sig.return_type, Type::Instance { name, .. } if name == "User"));

        // Test parse_obj()
        let parse_obj_sig = provider.get_method_signature(&user_type, "parse_obj");
        assert!(parse_obj_sig.is_some());
        let sig = parse_obj_sig.unwrap();
        assert!(matches!(sig.return_type, Type::Instance { name, .. } if name == "User"));
        assert_eq!(sig.params.len(), 1);

        // Test from_orm()
        let from_orm_sig = provider.get_method_signature(&user_type, "from_orm");
        assert!(from_orm_sig.is_some());
        let sig = from_orm_sig.unwrap();
        assert!(matches!(sig.return_type, Type::Instance { name, .. } if name == "User"));
    }

    #[test]
    fn test_django_queryset_methods() {
        let mut provider = DjangoTypeProvider::new();

        let mut fields = HashMap::new();
        fields.insert(
            "name".to_string(),
            DjangoField {
                name: "name".to_string(),
                field_type: DjangoFieldType::CharField,
                null: false,
                has_default: false,
            },
        );

        provider.register_model(DjangoModel {
            name: "User".to_string(),
            fields,
            relations: Vec::new(),
        });

        let queryset_type = Type::Instance {
            name: "UserQuerySet".to_string(),
            module: None,
            type_args: Vec::new(),
        };

        // Test filter() returns QuerySet
        let filter_sig = provider.get_method_signature(&queryset_type, "filter");
        assert!(filter_sig.is_some());
        let sig = filter_sig.unwrap();
        assert!(matches!(sig.return_type, Type::Instance { name, .. } if name == "UserQuerySet"));

        // Test get() returns Model instance
        let get_sig = provider.get_method_signature(&queryset_type, "get");
        assert!(get_sig.is_some());
        let sig = get_sig.unwrap();
        assert!(matches!(sig.return_type, Type::Instance { name, .. } if name == "User"));

        // Test count() returns int
        let count_sig = provider.get_method_signature(&queryset_type, "count");
        assert!(count_sig.is_some());
        let sig = count_sig.unwrap();
        assert!(matches!(sig.return_type, Type::Int));

        // Test first() returns Optional[Model]
        let first_sig = provider.get_method_signature(&queryset_type, "first");
        assert!(first_sig.is_some());
        let sig = first_sig.unwrap();
        assert!(matches!(sig.return_type, Type::Optional(..)));
    }
}
