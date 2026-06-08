//! Template rendering engine.
//!
//! Evaluates parsed template AST nodes against a context of variables.

use std::collections::HashMap;

use super::filters::apply_filter;
use super::parser::{BinOperator, Expr, Node};

/// A template value (dynamic type used during rendering).
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    String(String),
    Int(i64),
    Float(f64),
    Bool(bool),
    None,
    List(Vec<Value>),
    Map(HashMap<String, Value>),
}

impl Value {
    /// Check if value is truthy (Jinja2 semantics).
    pub fn is_truthy(&self) -> bool {
        match self {
            Value::Bool(b) => *b,
            Value::Int(i) => *i != 0,
            Value::Float(f) => *f != 0.0,
            Value::String(s) => !s.is_empty(),
            Value::None => false,
            Value::List(items) => !items.is_empty(),
            Value::Map(map) => !map.is_empty(),
        }
    }

    /// Convert value to display string.
    pub fn to_display_string(&self) -> String {
        match self {
            Value::String(s) => s.clone(),
            Value::Int(i) => i.to_string(),
            Value::Float(f) => {
                // Match Python/Jinja2 behavior for float display
                if *f == f.floor() && f.is_finite() {
                    format!("{:.1}", f)
                } else {
                    f.to_string()
                }
            }
            Value::Bool(b) => {
                if *b {
                    "True".to_string()
                } else {
                    "False".to_string()
                }
            }
            Value::None => "".to_string(),
            Value::List(items) => {
                let parts: Vec<String> = items.iter().map(|v| v.to_display_string()).collect();
                format!("[{}]", parts.join(", "))
            }
            Value::Map(map) => {
                let parts: Vec<String> = map
                    .iter()
                    .map(|(k, v)| format!("{}: {}", k, v.to_display_string()))
                    .collect();
                format!("{{{}}}", parts.join(", "))
            }
        }
    }
}

/// Convenience: Convert from serde_json::Value.
impl From<serde_json::Value> for Value {
    fn from(v: serde_json::Value) -> Self {
        match v {
            serde_json::Value::Null => Value::None,
            serde_json::Value::Bool(b) => Value::Bool(b),
            serde_json::Value::Number(n) => {
                if let Some(i) = n.as_i64() {
                    Value::Int(i)
                } else {
                    Value::Float(n.as_f64().unwrap_or(0.0))
                }
            }
            serde_json::Value::String(s) => Value::String(s),
            serde_json::Value::Array(arr) => {
                Value::List(arr.into_iter().map(Value::from).collect())
            }
            serde_json::Value::Object(obj) => {
                Value::Map(obj.into_iter().map(|(k, v)| (k, Value::from(v))).collect())
            }
        }
    }
}

/// Template rendering context (variable bindings).
#[derive(Debug, Clone)]
pub struct Context {
    scopes: Vec<HashMap<String, Value>>,
}

impl Default for Context {
    fn default() -> Self {
        Self::new()
    }
}

impl Context {
    /// Create a new empty context.
    pub fn new() -> Self {
        Self {
            scopes: vec![HashMap::new()],
        }
    }

    /// Create context from a HashMap.
    pub fn from_map(map: HashMap<String, Value>) -> Self {
        Self { scopes: vec![map] }
    }

    /// Set a variable in the current scope.
    pub fn set(&mut self, name: impl Into<String>, value: Value) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name.into(), value);
        }
    }

    /// Get a variable, searching from innermost scope outward.
    pub fn get(&self, name: &str) -> Option<&Value> {
        for scope in self.scopes.iter().rev() {
            if let Some(value) = scope.get(name) {
                return Some(value);
            }
        }
        None
    }

    /// Push a new scope (for blocks/loops).
    fn push_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    /// Pop the innermost scope.
    fn pop_scope(&mut self) {
        if self.scopes.len() > 1 {
            self.scopes.pop();
        }
    }
}

/// Template loader for template inheritance and includes.
pub trait TemplateLoader {
    /// Load a template by name (returns template source string).
    fn load(&self, name: &str) -> Option<String>;
}

/// File-system template loader.
pub struct FileLoader {
    base_dir: std::path::PathBuf,
}

impl FileLoader {
    pub fn new(base_dir: impl Into<std::path::PathBuf>) -> Self {
        Self {
            base_dir: base_dir.into(),
        }
    }
}

impl TemplateLoader for FileLoader {
    fn load(&self, name: &str) -> Option<String> {
        let path = self.base_dir.join(name);
        std::fs::read_to_string(path).ok()
    }
}

/// HashMap-based template loader (for testing or in-memory templates).
pub struct MapLoader {
    templates: HashMap<String, String>,
}

impl MapLoader {
    pub fn new() -> Self {
        Self {
            templates: HashMap::new(),
        }
    }

    pub fn add(&mut self, name: impl Into<String>, source: impl Into<String>) {
        self.templates.insert(name.into(), source.into());
    }
}

impl Default for MapLoader {
    fn default() -> Self {
        Self::new()
    }
}

impl TemplateLoader for MapLoader {
    fn load(&self, name: &str) -> Option<String> {
        self.templates.get(name).cloned()
    }
}

/// Template engine that renders parsed templates.
pub struct Engine {
    loader: Option<Box<dyn TemplateLoader>>,
}

impl Default for Engine {
    fn default() -> Self {
        Self::new()
    }
}

impl Engine {
    /// Create a new engine without a template loader.
    pub fn new() -> Self {
        Self { loader: None }
    }

    /// Create an engine with a template loader for inheritance/includes.
    pub fn with_loader(loader: Box<dyn TemplateLoader>) -> Self {
        Self {
            loader: Some(loader),
        }
    }

    /// Render a template string with a context.
    pub fn render(&self, template: &str, ctx: &mut Context) -> Result<String, String> {
        let nodes = super::parser::parse(template).map_err(|e| e.to_string())?;
        self.render_nodes(&nodes, ctx)
    }

    /// Render parsed nodes.
    fn render_nodes(&self, nodes: &[Node], ctx: &mut Context) -> Result<String, String> {
        // Check for extends directive
        let mut extends_name = None;
        let mut blocks: HashMap<String, Vec<Node>> = HashMap::new();

        for node in nodes {
            match node {
                Node::Extends(name) => {
                    extends_name = Some(name.clone());
                }
                Node::Block { name, body } => {
                    blocks.insert(name.clone(), body.clone());
                }
                _ => {}
            }
        }

        // Handle template inheritance
        if let Some(parent_name) = extends_name {
            return self.render_with_inheritance(&parent_name, &blocks, ctx);
        }

        let mut output = String::new();
        for node in nodes {
            output.push_str(&self.render_node(node, ctx, &blocks)?);
        }
        Ok(output)
    }

    /// Render with template inheritance.
    fn render_with_inheritance(
        &self,
        parent_name: &str,
        child_blocks: &HashMap<String, Vec<Node>>,
        ctx: &mut Context,
    ) -> Result<String, String> {
        let loader = self
            .loader
            .as_ref()
            .ok_or("Template loader required for extends")?;
        let parent_source = loader
            .load(parent_name)
            .ok_or(format!("Template not found: {}", parent_name))?;
        let parent_nodes = super::parser::parse(&parent_source).map_err(|e| e.to_string())?;

        let mut output = String::new();
        for node in &parent_nodes {
            output.push_str(&self.render_node(node, ctx, child_blocks)?);
        }
        Ok(output)
    }

    /// Render a single node.
    fn render_node(
        &self,
        node: &Node,
        ctx: &mut Context,
        blocks: &HashMap<String, Vec<Node>>,
    ) -> Result<String, String> {
        match node {
            Node::Text(text) => Ok(text.clone()),
            Node::Comment(_) => Ok(String::new()),
            Node::Raw(text) => Ok(text.clone()),
            Node::Expression(expr) => {
                let value = self.eval_expr(expr, ctx)?;
                Ok(value.to_display_string())
            }
            Node::If {
                condition,
                body,
                elif_branches,
                else_body,
            } => {
                let cond_val = self.eval_expr(condition, ctx)?;
                if cond_val.is_truthy() {
                    return self.render_node_list(body, ctx, blocks);
                }
                for (elif_cond, elif_body) in elif_branches {
                    let val = self.eval_expr(elif_cond, ctx)?;
                    if val.is_truthy() {
                        return self.render_node_list(elif_body, ctx, blocks);
                    }
                }
                self.render_node_list(else_body, ctx, blocks)
            }
            Node::For {
                variable,
                iterable,
                body,
                else_body,
            } => {
                let iter_val = self.eval_expr(iterable, ctx)?;
                match iter_val {
                    Value::List(items) if items.is_empty() => {
                        self.render_node_list(else_body, ctx, blocks)
                    }
                    Value::List(items) => {
                        let mut output = String::new();
                        let len = items.len();
                        for (index, item) in items.into_iter().enumerate() {
                            ctx.push_scope();
                            ctx.set(variable.clone(), item);
                            // Add loop variable
                            let mut loop_map = HashMap::new();
                            loop_map.insert("index0".to_string(), Value::Int(index as i64));
                            loop_map.insert("index".to_string(), Value::Int((index + 1) as i64));
                            loop_map.insert("first".to_string(), Value::Bool(index == 0));
                            loop_map.insert("last".to_string(), Value::Bool(index == len - 1));
                            loop_map.insert("length".to_string(), Value::Int(len as i64));
                            ctx.set("loop", Value::Map(loop_map));
                            output.push_str(&self.render_node_list(body, ctx, blocks)?);
                            ctx.pop_scope();
                        }
                        Ok(output)
                    }
                    _ => Err(format!("Cannot iterate over: {:?}", iter_val)),
                }
            }
            Node::Block { name, body } => {
                // Check if child template overrides this block
                if let Some(override_body) = blocks.get(name) {
                    self.render_node_list(override_body, ctx, blocks)
                } else {
                    self.render_node_list(body, ctx, blocks)
                }
            }
            Node::Set { name, value } => {
                let val = self.eval_expr(value, ctx)?;
                ctx.set(name.clone(), val);
                Ok(String::new())
            }
            Node::Include(name) => {
                let loader = self
                    .loader
                    .as_ref()
                    .ok_or("Template loader required for include")?;
                let source = loader
                    .load(name)
                    .ok_or(format!("Template not found: {}", name))?;
                self.render(&source, ctx)
            }
            Node::Extends(_) => Ok(String::new()), // Handled at render_nodes level
        }
    }

    fn render_node_list(
        &self,
        nodes: &[Node],
        ctx: &mut Context,
        blocks: &HashMap<String, Vec<Node>>,
    ) -> Result<String, String> {
        let mut output = String::new();
        for node in nodes {
            output.push_str(&self.render_node(node, ctx, blocks)?);
        }
        Ok(output)
    }

    /// Evaluate an expression in the given context.
    fn eval_expr(&self, expr: &Expr, ctx: &mut Context) -> Result<Value, String> {
        match expr {
            Expr::StringLit(s) => Ok(Value::String(s.clone())),
            Expr::IntLit(i) => Ok(Value::Int(*i)),
            Expr::FloatLit(f) => Ok(Value::Float(*f)),
            Expr::BoolLit(b) => Ok(Value::Bool(*b)),
            Expr::NoneLit => Ok(Value::None),
            Expr::Variable(name) => Ok(ctx.get(name).cloned().unwrap_or(Value::None)),
            Expr::Attribute(obj_expr, attr) => {
                let obj = self.eval_expr(obj_expr, ctx)?;
                match obj {
                    Value::Map(map) => Ok(map.get(attr).cloned().unwrap_or(Value::None)),
                    _ => Ok(Value::None),
                }
            }
            Expr::Index(obj_expr, idx_expr) => {
                let obj = self.eval_expr(obj_expr, ctx)?;
                let idx = self.eval_expr(idx_expr, ctx)?;
                match (&obj, &idx) {
                    (Value::List(items), Value::Int(i)) => {
                        let index = if *i < 0 {
                            (items.len() as i64 + *i) as usize
                        } else {
                            *i as usize
                        };
                        Ok(items.get(index).cloned().unwrap_or(Value::None))
                    }
                    (Value::Map(map), Value::String(key)) => {
                        Ok(map.get(key).cloned().unwrap_or(Value::None))
                    }
                    _ => Ok(Value::None),
                }
            }
            Expr::Filter { expr, name, args } => {
                let value = self.eval_expr(expr, ctx)?;
                let eval_args: Vec<Value> = args
                    .iter()
                    .map(|a| self.eval_expr(a, ctx))
                    .collect::<Result<_, _>>()?;
                apply_filter(name, &value, &eval_args)
            }
            Expr::BinOp { left, op, right } => {
                let lval = self.eval_expr(left, ctx)?;
                let rval = self.eval_expr(right, ctx)?;
                self.eval_binop(&lval, op, &rval)
            }
            Expr::Not(inner) => {
                let val = self.eval_expr(inner, ctx)?;
                Ok(Value::Bool(!val.is_truthy()))
            }
            Expr::List(items) => {
                let values: Vec<Value> = items
                    .iter()
                    .map(|e| self.eval_expr(e, ctx))
                    .collect::<Result<_, _>>()?;
                Ok(Value::List(values))
            }
        }
    }

    /// Evaluate a binary operation.
    fn eval_binop(&self, left: &Value, op: &BinOperator, right: &Value) -> Result<Value, String> {
        match op {
            BinOperator::Add => match (left, right) {
                (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a + b)),
                (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a + b)),
                (Value::Int(a), Value::Float(b)) => Ok(Value::Float(*a as f64 + b)),
                (Value::Float(a), Value::Int(b)) => Ok(Value::Float(a + *b as f64)),
                (Value::String(a), Value::String(b)) => Ok(Value::String(format!("{}{}", a, b))),
                _ => Err(format!("Cannot add {:?} and {:?}", left, right)),
            },
            BinOperator::Sub => match (left, right) {
                (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a - b)),
                (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a - b)),
                (Value::Int(a), Value::Float(b)) => Ok(Value::Float(*a as f64 - b)),
                (Value::Float(a), Value::Int(b)) => Ok(Value::Float(a - *b as f64)),
                _ => Err(format!("Cannot subtract {:?} and {:?}", left, right)),
            },
            BinOperator::Mul => match (left, right) {
                (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a * b)),
                (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a * b)),
                (Value::Int(a), Value::Float(b)) => Ok(Value::Float(*a as f64 * b)),
                (Value::Float(a), Value::Int(b)) => Ok(Value::Float(a * *b as f64)),
                (Value::String(s), Value::Int(n)) => Ok(Value::String(s.repeat(*n as usize))),
                _ => Err(format!("Cannot multiply {:?} and {:?}", left, right)),
            },
            BinOperator::Div => {
                let a = match left {
                    Value::Int(i) => *i as f64,
                    Value::Float(f) => *f,
                    _ => return Err(format!("Cannot divide {:?}", left)),
                };
                let b = match right {
                    Value::Int(i) => *i as f64,
                    Value::Float(f) => *f,
                    _ => return Err(format!("Cannot divide by {:?}", right)),
                };
                if b == 0.0 {
                    return Err("Division by zero".to_string());
                }
                Ok(Value::Float(a / b))
            }
            BinOperator::Mod => match (left, right) {
                (Value::Int(a), Value::Int(b)) => {
                    if *b == 0 {
                        Err("Modulo by zero".to_string())
                    } else {
                        Ok(Value::Int(a % b))
                    }
                }
                _ => Err(format!("Cannot modulo {:?} and {:?}", left, right)),
            },
            BinOperator::Eq => Ok(Value::Bool(left == right)),
            BinOperator::Ne => Ok(Value::Bool(left != right)),
            BinOperator::Lt => Ok(Value::Bool(
                compare_ord(left, right) == std::cmp::Ordering::Less,
            )),
            BinOperator::Gt => Ok(Value::Bool(
                compare_ord(left, right) == std::cmp::Ordering::Greater,
            )),
            BinOperator::Le => Ok(Value::Bool(
                compare_ord(left, right) != std::cmp::Ordering::Greater,
            )),
            BinOperator::Ge => Ok(Value::Bool(
                compare_ord(left, right) != std::cmp::Ordering::Less,
            )),
            BinOperator::And => {
                if left.is_truthy() {
                    Ok(right.clone())
                } else {
                    Ok(left.clone())
                }
            }
            BinOperator::Or => {
                if left.is_truthy() {
                    Ok(left.clone())
                } else {
                    Ok(right.clone())
                }
            }
            BinOperator::In => match right {
                Value::List(items) => Ok(Value::Bool(items.contains(left))),
                Value::String(s) => {
                    let needle = match left {
                        Value::String(n) => n.clone(),
                        _ => left.to_display_string(),
                    };
                    Ok(Value::Bool(s.contains(&needle)))
                }
                Value::Map(map) => {
                    let key = match left {
                        Value::String(s) => s.clone(),
                        _ => left.to_display_string(),
                    };
                    Ok(Value::Bool(map.contains_key(&key)))
                }
                _ => Ok(Value::Bool(false)),
            },
        }
    }
}

fn compare_ord(a: &Value, b: &Value) -> std::cmp::Ordering {
    match (a, b) {
        (Value::Int(a), Value::Int(b)) => a.cmp(b),
        (Value::Float(a), Value::Float(b)) => a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal),
        (Value::Int(a), Value::Float(b)) => (*a as f64)
            .partial_cmp(b)
            .unwrap_or(std::cmp::Ordering::Equal),
        (Value::Float(a), Value::Int(b)) => a
            .partial_cmp(&(*b as f64))
            .unwrap_or(std::cmp::Ordering::Equal),
        (Value::String(a), Value::String(b)) => a.cmp(b),
        _ => std::cmp::Ordering::Equal,
    }
}

/// Convenience function: render a template string with a JSON-like context.
pub fn render(template: &str, vars: HashMap<String, Value>) -> Result<String, String> {
    let mut ctx = Context::from_map(vars);
    Engine::new().render(template, &mut ctx)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_text() {
        let result = render("Hello, World!", HashMap::new()).unwrap();
        assert_eq!(result, "Hello, World!");
    }

    #[test]
    fn test_render_variable() {
        let mut vars = HashMap::new();
        vars.insert("name".to_string(), Value::String("Alice".to_string()));
        let result = render("Hello, {{ name }}!", vars).unwrap();
        assert_eq!(result, "Hello, Alice!");
    }

    #[test]
    fn test_render_if_true() {
        let mut vars = HashMap::new();
        vars.insert("show".to_string(), Value::Bool(true));
        let result = render("{% if show %}visible{% endif %}", vars).unwrap();
        assert_eq!(result, "visible");
    }

    #[test]
    fn test_render_if_false() {
        let mut vars = HashMap::new();
        vars.insert("show".to_string(), Value::Bool(false));
        let result = render("{% if show %}visible{% else %}hidden{% endif %}", vars).unwrap();
        assert_eq!(result, "hidden");
    }

    #[test]
    fn test_render_for_loop() {
        let mut vars = HashMap::new();
        vars.insert(
            "items".to_string(),
            Value::List(vec![
                Value::String("a".to_string()),
                Value::String("b".to_string()),
                Value::String("c".to_string()),
            ]),
        );
        let result = render("{% for item in items %}{{ item }}{% endfor %}", vars).unwrap();
        assert_eq!(result, "abc");
    }

    #[test]
    fn test_render_filter() {
        let mut vars = HashMap::new();
        vars.insert("name".to_string(), Value::String("alice".to_string()));
        let result = render("{{ name | upper }}", vars).unwrap();
        assert_eq!(result, "ALICE");
    }

    #[test]
    fn test_render_arithmetic() {
        let mut vars = HashMap::new();
        vars.insert("x".to_string(), Value::Int(10));
        vars.insert("y".to_string(), Value::Int(3));
        let result = render("{{ x + y }}", vars).unwrap();
        assert_eq!(result, "13");
    }

    #[test]
    fn test_render_set() {
        let result = render("{% set x = 42 %}{{ x }}", HashMap::new()).unwrap();
        assert_eq!(result, "42");
    }

    #[test]
    fn test_loop_variable() {
        let mut vars = HashMap::new();
        vars.insert(
            "items".to_string(),
            Value::List(vec![
                Value::String("a".to_string()),
                Value::String("b".to_string()),
            ]),
        );
        let result = render("{% for item in items %}{{ loop.index }}{% endfor %}", vars).unwrap();
        assert_eq!(result, "12");
    }

    #[test]
    fn test_template_inheritance() {
        let mut loader = MapLoader::new();
        loader.add(
            "base.html",
            "Header{% block content %}default{% endblock %}Footer",
        );

        let engine = Engine::with_loader(Box::new(loader));
        let mut ctx = Context::new();
        let result = engine
            .render(
                "{% extends \"base.html\" %}{% block content %}custom{% endblock %}",
                &mut ctx,
            )
            .unwrap();
        assert_eq!(result, "HeadercustomFooter");
    }
}
