//! Reusable dependency injection core for `mambalibs.di`.
//!
//! This crate intentionally has no HTTP dependency. Web frameworks can adapt
//! request parameters to `ProviderKey`, but provider registration, scope
//! caching, and test overrides live here.

use std::any::{type_name, Any};
use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use thiserror::Error;

pub type DiResult<T> = Result<T, DiError>;
pub type DiValue = Arc<dyn Any + Send + Sync>;

type ProviderFactory = Arc<dyn for<'a> Fn(&Resolver<'a>) -> DiResult<DiValue> + Send + Sync>;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum DiError {
    #[error("provider key must not be empty")]
    EmptyKey,
    #[error("provider not found: {0}")]
    NotFound(String),
    #[error("provider {0} requires a request scope")]
    RequestScopeRequired(String),
    #[error("provider {key} returned a value that is not {expected}")]
    TypeMismatch { key: String, expected: &'static str },
    #[error("cyclic dependency detected: {0}")]
    CyclicDependency(String),
    #[error("DI lock poisoned: {0}")]
    LockPoisoned(&'static str),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ProviderKey(String);

impl ProviderKey {
    pub fn new(name: impl Into<String>) -> DiResult<Self> {
        let name = name.into();
        let trimmed = name.trim();
        if trimmed.is_empty() {
            return Err(DiError::EmptyKey);
        }
        Ok(Self(trimmed.to_string()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<&str> for ProviderKey {
    type Error = DiError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl TryFrom<String> for ProviderKey {
    type Error = DiError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScopeKind {
    Singleton,
    Request,
    Transient,
}

impl ScopeKind {
    pub fn parse(value: &str) -> Option<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "singleton" => Some(Self::Singleton),
            "request" => Some(Self::Request),
            "transient" => Some(Self::Transient),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Singleton => "singleton",
            Self::Request => "request",
            Self::Transient => "transient",
        }
    }
}

#[derive(Clone)]
struct ProviderEntry {
    scope: ScopeKind,
    factory: ProviderFactory,
}

#[derive(Clone, Default)]
pub struct Container {
    inner: Arc<ContainerInner>,
}

#[derive(Default)]
struct ContainerInner {
    providers: RwLock<HashMap<ProviderKey, ProviderEntry>>,
    overrides: RwLock<HashMap<ProviderKey, DiValue>>,
    singletons: RwLock<HashMap<ProviderKey, DiValue>>,
}

impl Container {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register_factory<T, F>(
        &self,
        key: impl TryInto<ProviderKey, Error = DiError>,
        scope: ScopeKind,
        factory: F,
    ) -> DiResult<()>
    where
        T: Any + Send + Sync + 'static,
        F: for<'a> Fn(&Resolver<'a>) -> DiResult<T> + Send + Sync + 'static,
    {
        let key = key.try_into()?;
        let entry = ProviderEntry {
            scope,
            factory: Arc::new(move |resolver| Ok(Arc::new(factory(resolver)?) as DiValue)),
        };
        self.inner
            .providers
            .write()
            .map_err(|_| DiError::LockPoisoned("providers"))?
            .insert(key, entry);
        Ok(())
    }

    pub fn register_value<T>(
        &self,
        key: impl TryInto<ProviderKey, Error = DiError>,
        scope: ScopeKind,
        value: T,
    ) -> DiResult<()>
    where
        T: Any + Send + Sync + Clone + 'static,
    {
        self.register_factory(key, scope, move |_| Ok(value.clone()))
    }

    pub fn override_value<T>(
        &self,
        key: impl TryInto<ProviderKey, Error = DiError>,
        value: T,
    ) -> DiResult<()>
    where
        T: Any + Send + Sync + 'static,
    {
        let key = key.try_into()?;
        self.inner
            .overrides
            .write()
            .map_err(|_| DiError::LockPoisoned("overrides"))?
            .insert(key, Arc::new(value));
        Ok(())
    }

    pub fn clear_override(&self, key: impl TryInto<ProviderKey, Error = DiError>) -> DiResult<()> {
        let key = key.try_into()?;
        self.inner
            .overrides
            .write()
            .map_err(|_| DiError::LockPoisoned("overrides"))?
            .remove(&key);
        Ok(())
    }

    pub fn request_scope(&self) -> RequestScope {
        RequestScope {
            container: self.clone(),
            cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn resolve<T>(&self, key: impl TryInto<ProviderKey, Error = DiError>) -> DiResult<Arc<T>>
    where
        T: Any + Send + Sync + 'static,
    {
        let key = key.try_into()?;
        let resolver = Resolver {
            container: self.clone(),
            request_cache: None,
            resolution_stack: RefCell::new(Vec::new()),
        };
        resolver.resolve_typed(&key)
    }

    pub fn resolve_any(
        &self,
        key: impl TryInto<ProviderKey, Error = DiError>,
    ) -> DiResult<DiValue> {
        let key = key.try_into()?;
        let resolver = Resolver {
            container: self.clone(),
            request_cache: None,
            resolution_stack: RefCell::new(Vec::new()),
        };
        resolver.resolve_any_key(&key)
    }

    pub fn resolve_many_any<I, K>(&self, keys: I) -> DiResult<HashMap<String, DiValue>>
    where
        I: IntoIterator<Item = K>,
        K: TryInto<ProviderKey, Error = DiError>,
    {
        let resolver = Resolver {
            container: self.clone(),
            request_cache: None,
            resolution_stack: RefCell::new(Vec::new()),
        };
        resolver.resolve_many_any(keys)
    }

    pub fn resolve_many<T, I, K>(&self, keys: I) -> DiResult<HashMap<String, Arc<T>>>
    where
        T: Any + Send + Sync + 'static,
        I: IntoIterator<Item = K>,
        K: TryInto<ProviderKey, Error = DiError>,
    {
        let resolver = Resolver {
            container: self.clone(),
            request_cache: None,
            resolution_stack: RefCell::new(Vec::new()),
        };
        resolver.resolve_many(keys)
    }
}

#[derive(Clone)]
pub struct RequestScope {
    container: Container,
    cache: Arc<RwLock<HashMap<ProviderKey, DiValue>>>,
}

impl RequestScope {
    pub fn resolve<T>(&self, key: impl TryInto<ProviderKey, Error = DiError>) -> DiResult<Arc<T>>
    where
        T: Any + Send + Sync + 'static,
    {
        let key = key.try_into()?;
        let resolver = Resolver {
            container: self.container.clone(),
            request_cache: Some(&self.cache),
            resolution_stack: RefCell::new(Vec::new()),
        };
        resolver.resolve_typed(&key)
    }

    pub fn resolve_any(
        &self,
        key: impl TryInto<ProviderKey, Error = DiError>,
    ) -> DiResult<DiValue> {
        let key = key.try_into()?;
        let resolver = Resolver {
            container: self.container.clone(),
            request_cache: Some(&self.cache),
            resolution_stack: RefCell::new(Vec::new()),
        };
        resolver.resolve_any_key(&key)
    }

    pub fn resolve_many_any<I, K>(&self, keys: I) -> DiResult<HashMap<String, DiValue>>
    where
        I: IntoIterator<Item = K>,
        K: TryInto<ProviderKey, Error = DiError>,
    {
        let resolver = Resolver {
            container: self.container.clone(),
            request_cache: Some(&self.cache),
            resolution_stack: RefCell::new(Vec::new()),
        };
        resolver.resolve_many_any(keys)
    }

    pub fn resolve_many<T, I, K>(&self, keys: I) -> DiResult<HashMap<String, Arc<T>>>
    where
        T: Any + Send + Sync + 'static,
        I: IntoIterator<Item = K>,
        K: TryInto<ProviderKey, Error = DiError>,
    {
        let resolver = Resolver {
            container: self.container.clone(),
            request_cache: Some(&self.cache),
            resolution_stack: RefCell::new(Vec::new()),
        };
        resolver.resolve_many(keys)
    }
}

pub struct Resolver<'a> {
    container: Container,
    request_cache: Option<&'a RwLock<HashMap<ProviderKey, DiValue>>>,
    resolution_stack: RefCell<Vec<ProviderKey>>,
}

impl Resolver<'_> {
    pub fn resolve<T>(&self, key: impl TryInto<ProviderKey, Error = DiError>) -> DiResult<Arc<T>>
    where
        T: Any + Send + Sync + 'static,
    {
        let key = key.try_into()?;
        self.resolve_typed(&key)
    }

    pub fn resolve_any(
        &self,
        key: impl TryInto<ProviderKey, Error = DiError>,
    ) -> DiResult<DiValue> {
        let key = key.try_into()?;
        self.resolve_any_key(&key)
    }

    pub fn resolve_many_any<I, K>(&self, keys: I) -> DiResult<HashMap<String, DiValue>>
    where
        I: IntoIterator<Item = K>,
        K: TryInto<ProviderKey, Error = DiError>,
    {
        let mut resolved = HashMap::new();
        for key in keys {
            let key = key.try_into()?;
            let value = self.resolve_any_key(&key)?;
            resolved.insert(key.as_str().to_string(), value);
        }
        Ok(resolved)
    }

    pub fn resolve_many<T, I, K>(&self, keys: I) -> DiResult<HashMap<String, Arc<T>>>
    where
        T: Any + Send + Sync + 'static,
        I: IntoIterator<Item = K>,
        K: TryInto<ProviderKey, Error = DiError>,
    {
        let values = self.resolve_many_any(keys)?;
        values
            .into_iter()
            .map(|(key, value)| {
                value
                    .downcast::<T>()
                    .map(|value| (key.clone(), value))
                    .map_err(|_| DiError::TypeMismatch {
                        key,
                        expected: type_name::<T>(),
                    })
            })
            .collect()
    }

    fn resolve_typed<T>(&self, key: &ProviderKey) -> DiResult<Arc<T>>
    where
        T: Any + Send + Sync + 'static,
    {
        let value = self.resolve_any_key(key)?;
        value.downcast::<T>().map_err(|_| DiError::TypeMismatch {
            key: key.as_str().to_string(),
            expected: type_name::<T>(),
        })
    }

    fn resolve_any_key(&self, key: &ProviderKey) -> DiResult<DiValue> {
        let _guard = self.enter_resolution(key)?;
        self.resolve_any_key_inner(key)
    }

    fn resolve_any_key_inner(&self, key: &ProviderKey) -> DiResult<DiValue> {
        if let Some(value) = self
            .container
            .inner
            .overrides
            .read()
            .map_err(|_| DiError::LockPoisoned("overrides"))?
            .get(key)
            .cloned()
        {
            return Ok(value);
        }

        let entry = self
            .container
            .inner
            .providers
            .read()
            .map_err(|_| DiError::LockPoisoned("providers"))?
            .get(key)
            .cloned()
            .ok_or_else(|| DiError::NotFound(key.as_str().to_string()))?;

        match entry.scope {
            ScopeKind::Singleton => self.resolve_singleton(key, &entry),
            ScopeKind::Request => self.resolve_request(key, &entry),
            ScopeKind::Transient => (entry.factory)(self),
        }
    }

    fn resolve_singleton(&self, key: &ProviderKey, entry: &ProviderEntry) -> DiResult<DiValue> {
        if let Some(value) = self
            .container
            .inner
            .singletons
            .read()
            .map_err(|_| DiError::LockPoisoned("singletons"))?
            .get(key)
            .cloned()
        {
            return Ok(value);
        }

        let value = (entry.factory)(self)?;
        self.container
            .inner
            .singletons
            .write()
            .map_err(|_| DiError::LockPoisoned("singletons"))?
            .insert(key.clone(), value.clone());
        Ok(value)
    }

    fn resolve_request(&self, key: &ProviderKey, entry: &ProviderEntry) -> DiResult<DiValue> {
        let cache = self
            .request_cache
            .ok_or_else(|| DiError::RequestScopeRequired(key.as_str().to_string()))?;

        if let Some(value) = cache
            .read()
            .map_err(|_| DiError::LockPoisoned("request_cache"))?
            .get(key)
            .cloned()
        {
            return Ok(value);
        }

        let value = (entry.factory)(self)?;
        cache
            .write()
            .map_err(|_| DiError::LockPoisoned("request_cache"))?
            .insert(key.clone(), value.clone());
        Ok(value)
    }

    fn enter_resolution<'a>(&'a self, key: &ProviderKey) -> DiResult<ResolutionGuard<'a>> {
        let mut stack = self.resolution_stack.borrow_mut();
        if let Some(first_seen) = stack.iter().position(|candidate| candidate == key) {
            let mut chain = stack[first_seen..]
                .iter()
                .map(|key| key.as_str().to_string())
                .collect::<Vec<_>>();
            chain.push(key.as_str().to_string());
            return Err(DiError::CyclicDependency(chain.join(" -> ")));
        }
        stack.push(key.clone());
        Ok(ResolutionGuard {
            stack: &self.resolution_stack,
        })
    }
}

struct ResolutionGuard<'a> {
    stack: &'a RefCell<Vec<ProviderKey>>,
}

impl Drop for ResolutionGuard<'_> {
    fn drop(&mut self) {
        let _ = self.stack.borrow_mut().pop();
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DependencyMarker {
    key: Option<ProviderKey>,
}

impl DependencyMarker {
    pub fn inferred() -> Self {
        Self { key: None }
    }

    pub fn new(key: impl TryInto<ProviderKey, Error = DiError>) -> DiResult<Self> {
        Ok(Self {
            key: Some(key.try_into()?),
        })
    }

    pub fn key(&self) -> Option<&ProviderKey> {
        self.key.as_ref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[test]
    fn singleton_provider_is_cached() {
        let container = Container::new();
        let calls = Arc::new(AtomicUsize::new(0));
        let calls_for_factory = calls.clone();
        container
            .register_factory("config", ScopeKind::Singleton, move |_| {
                let n = calls_for_factory.fetch_add(1, Ordering::SeqCst) + 1;
                Ok(format!("config-{n}"))
            })
            .unwrap();

        let a = container.resolve::<String>("config").unwrap();
        let b = container.resolve::<String>("config").unwrap();

        assert_eq!(calls.load(Ordering::SeqCst), 1);
        assert_eq!(a.as_str(), "config-1");
        assert!(Arc::ptr_eq(&a, &b));
    }

    #[test]
    fn request_scope_caches_per_scope() {
        let container = Container::new();
        let calls = Arc::new(AtomicUsize::new(0));
        let calls_for_factory = calls.clone();
        container
            .register_factory("db", ScopeKind::Request, move |_| {
                Ok(calls_for_factory.fetch_add(1, Ordering::SeqCst) + 1)
            })
            .unwrap();

        let scope_a = container.request_scope();
        let scope_b = container.request_scope();

        let a1 = scope_a.resolve::<usize>("db").unwrap();
        let a2 = scope_a.resolve::<usize>("db").unwrap();
        let b1 = scope_b.resolve::<usize>("db").unwrap();

        assert_eq!(*a1, 1);
        assert_eq!(*a2, 1);
        assert!(Arc::ptr_eq(&a1, &a2));
        assert_eq!(*b1, 2);
        assert_eq!(calls.load(Ordering::SeqCst), 2);
    }

    #[test]
    fn request_provider_requires_request_scope() {
        let container = Container::new();
        container
            .register_value("db", ScopeKind::Request, "session".to_string())
            .unwrap();

        let error = container.resolve::<String>("db").unwrap_err();
        assert_eq!(error, DiError::RequestScopeRequired("db".to_string()));
    }

    #[test]
    fn transient_provider_is_not_cached() {
        let container = Container::new();
        let calls = Arc::new(AtomicUsize::new(0));
        let calls_for_factory = calls.clone();
        container
            .register_factory("token", ScopeKind::Transient, move |_| {
                Ok(calls_for_factory.fetch_add(1, Ordering::SeqCst) + 1)
            })
            .unwrap();

        let first = container.resolve::<usize>("token").unwrap();
        let second = container.resolve::<usize>("token").unwrap();

        assert_eq!(*first, 1);
        assert_eq!(*second, 2);
        assert_eq!(calls.load(Ordering::SeqCst), 2);
    }

    #[test]
    fn override_wins_before_provider() {
        let container = Container::new();
        let calls = Arc::new(AtomicUsize::new(0));
        let calls_for_factory = calls.clone();
        container
            .register_factory("client", ScopeKind::Singleton, move |_| {
                calls_for_factory.fetch_add(1, Ordering::SeqCst);
                Ok("real".to_string())
            })
            .unwrap();
        container
            .override_value("client", "fake".to_string())
            .unwrap();

        let resolved = container.resolve::<String>("client").unwrap();

        assert_eq!(resolved.as_str(), "fake");
        assert_eq!(calls.load(Ordering::SeqCst), 0);
    }

    #[test]
    fn container_resolve_many_returns_keyed_typed_values() {
        let container = Container::new();
        container
            .register_value("settings", ScopeKind::Singleton, "prod".to_string())
            .unwrap();
        container
            .register_value("client", ScopeKind::Singleton, "http".to_string())
            .unwrap();

        let resolved = container
            .resolve_many::<String, _, _>(["settings", "client"])
            .unwrap();

        assert_eq!(resolved.len(), 2);
        assert_eq!(resolved["settings"].as_str(), "prod");
        assert_eq!(resolved["client"].as_str(), "http");
    }

    #[test]
    fn request_scope_resolve_many_reuses_request_cache() {
        let container = Container::new();
        let calls = Arc::new(AtomicUsize::new(0));
        let calls_for_factory = calls.clone();
        container
            .register_factory("db", ScopeKind::Request, move |_| {
                Ok(calls_for_factory.fetch_add(1, Ordering::SeqCst) + 1)
            })
            .unwrap();
        let scope = container.request_scope();

        let resolved = scope.resolve_many::<usize, _, _>(["db", "db"]).unwrap();

        assert_eq!(resolved.len(), 1);
        assert_eq!(*resolved["db"], 1);
        assert_eq!(calls.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn cyclic_dependency_returns_error_instead_of_recursing() {
        let container = Container::new();
        container
            .register_factory("settings", ScopeKind::Transient, |resolver| {
                resolver
                    .resolve::<String>("client")
                    .map(|_| "settings".to_string())
            })
            .unwrap();
        container
            .register_factory("client", ScopeKind::Transient, |resolver| {
                resolver
                    .resolve::<String>("settings")
                    .map(|_| "client".to_string())
            })
            .unwrap();

        let error = container.resolve::<String>("settings").unwrap_err();

        assert_eq!(
            error,
            DiError::CyclicDependency("settings -> client -> settings".to_string())
        );
    }

    #[test]
    fn dependency_marker_records_optional_key() {
        let inferred = DependencyMarker::inferred();
        assert!(inferred.key().is_none());

        let marker = DependencyMarker::new("current_user").unwrap();
        assert_eq!(marker.key().map(ProviderKey::as_str), Some("current_user"));
    }
}
