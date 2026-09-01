//! Weighted quota admission with an RAII concurrency lease.

use std::{
    collections::HashMap,
    fmt,
    hash::Hash,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WeightedAdmissionConfig {
    pub max_concurrent_per_key: usize,
    pub max_weight_per_window: usize,
    pub window: Duration,
    pub max_keys: usize,
}

impl WeightedAdmissionConfig {
    pub fn new(
        max_concurrent_per_key: usize,
        max_weight_per_window: usize,
        window: Duration,
        max_keys: usize,
    ) -> Result<Self, WeightedAdmissionConfigError> {
        if max_concurrent_per_key == 0 {
            return Err(WeightedAdmissionConfigError::ZeroConcurrency);
        }
        if max_weight_per_window == 0 {
            return Err(WeightedAdmissionConfigError::ZeroWeight);
        }
        if window.is_zero() {
            return Err(WeightedAdmissionConfigError::ZeroWindow);
        }
        if max_keys == 0 {
            return Err(WeightedAdmissionConfigError::ZeroKeys);
        }
        Ok(Self {
            max_concurrent_per_key,
            max_weight_per_window,
            window,
            max_keys,
        })
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, thiserror::Error)]
pub enum WeightedAdmissionConfigError {
    #[error("weighted admission concurrency must be positive")]
    ZeroConcurrency,
    #[error("weighted admission window limit must be positive")]
    ZeroWeight,
    #[error("weighted admission window must be positive")]
    ZeroWindow,
    #[error("weighted admission key limit must be positive")]
    ZeroKeys,
}

#[derive(Debug, thiserror::Error)]
pub enum WeightedAdmissionError {
    #[error("service is draining")]
    Draining,
    #[error("request weight must be positive")]
    ZeroWeight,
    #[error("admission key capacity is exhausted")]
    KeyLimitExceeded,
    #[error("admission concurrency limit {limit} exceeded")]
    ConcurrencyExceeded { limit: usize },
    #[error("admission quota {limit} exceeded for the current window")]
    QuotaExceeded { limit: usize, retry_after: Duration },
}

impl WeightedAdmissionError {
    pub fn retry_after(&self) -> Option<Duration> {
        match self {
            Self::Draining | Self::ConcurrencyExceeded { .. } => Some(Duration::from_secs(1)),
            Self::QuotaExceeded { retry_after, .. } => Some(*retry_after),
            Self::ZeroWeight | Self::KeyLimitExceeded => None,
        }
    }
}

#[derive(Debug)]
struct KeyState {
    in_flight: usize,
    used_weight: usize,
    window_started: Duration,
    last_seen: u64,
}

#[derive(Debug)]
struct AdmissionState<K> {
    keys: HashMap<K, KeyState>,
    sequence: u64,
}

impl<K> Default for AdmissionState<K> {
    fn default() -> Self {
        Self {
            keys: HashMap::new(),
            sequence: 0,
        }
    }
}

struct WeightedAdmissionInner<K> {
    config: WeightedAdmissionConfig,
    state: Mutex<AdmissionState<K>>,
    epoch: Instant,
}

pub struct WeightedAdmission<K>(Arc<WeightedAdmissionInner<K>>);

impl<K> Clone for WeightedAdmission<K> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<K> fmt::Debug for WeightedAdmission<K> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("WeightedAdmission")
            .field("config", &self.0.config)
            .finish_non_exhaustive()
    }
}

impl<K> WeightedAdmission<K>
where
    K: Clone + Eq + Hash + Send + 'static,
{
    pub fn new(config: WeightedAdmissionConfig) -> Self {
        Self(Arc::new(WeightedAdmissionInner {
            config,
            state: Mutex::new(AdmissionState::default()),
            epoch: Instant::now(),
        }))
    }

    pub fn acquire(
        &self,
        key: K,
        weight: usize,
        draining: bool,
    ) -> Result<ConcurrencyLease<K>, WeightedAdmissionError> {
        self.acquire_at(key, weight, draining, self.0.epoch.elapsed())
    }

    pub fn acquire_at(
        &self,
        key: K,
        weight: usize,
        draining: bool,
        now: Duration,
    ) -> Result<ConcurrencyLease<K>, WeightedAdmissionError> {
        if draining {
            return Err(WeightedAdmissionError::Draining);
        }
        if weight == 0 {
            return Err(WeightedAdmissionError::ZeroWeight);
        }
        let mut state = self
            .0
            .state
            .lock()
            .expect("weighted admission lock poisoned");
        state.sequence = state.sequence.wrapping_add(1);
        let sequence = state.sequence;
        if !state.keys.contains_key(&key) && state.keys.len() >= self.0.config.max_keys {
            let inactive = state
                .keys
                .iter()
                .filter(|(_, state)| state.in_flight == 0)
                .min_by_key(|(_, state)| state.last_seen)
                .map(|(key, _)| key.clone());
            match inactive {
                Some(inactive) => {
                    state.keys.remove(&inactive);
                }
                None => return Err(WeightedAdmissionError::KeyLimitExceeded),
            }
        }
        let key_state = state.keys.entry(key.clone()).or_insert(KeyState {
            in_flight: 0,
            used_weight: 0,
            window_started: now,
            last_seen: sequence,
        });
        if now.saturating_sub(key_state.window_started) >= self.0.config.window {
            key_state.used_weight = 0;
            key_state.window_started = now;
        }
        key_state.last_seen = sequence;
        if key_state.in_flight >= self.0.config.max_concurrent_per_key {
            return Err(WeightedAdmissionError::ConcurrencyExceeded {
                limit: self.0.config.max_concurrent_per_key,
            });
        }
        if key_state.used_weight.saturating_add(weight) > self.0.config.max_weight_per_window {
            return Err(WeightedAdmissionError::QuotaExceeded {
                limit: self.0.config.max_weight_per_window,
                retry_after: self
                    .0
                    .config
                    .window
                    .saturating_sub(now.saturating_sub(key_state.window_started))
                    .max(Duration::from_nanos(1)),
            });
        }
        key_state.in_flight += 1;
        key_state.used_weight += weight;
        Ok(ConcurrencyLease {
            key: Some(key),
            admission: self.clone(),
        })
    }
}

/// A concurrency slot that releases itself on every return path.
pub struct ConcurrencyLease<K>
where
    K: Clone + Eq + Hash + Send + 'static,
{
    key: Option<K>,
    admission: WeightedAdmission<K>,
}

impl<K> fmt::Debug for ConcurrencyLease<K>
where
    K: Clone + Eq + Hash + Send + 'static,
{
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ConcurrencyLease")
            .field("active", &self.key.is_some())
            .finish()
    }
}

impl<K> Drop for ConcurrencyLease<K>
where
    K: Clone + Eq + Hash + Send + 'static,
{
    fn drop(&mut self) {
        let Some(key) = self.key.take() else {
            return;
        };
        if let Some(state) = self
            .admission
            .0
            .state
            .lock()
            .expect("weighted admission lock poisoned")
            .keys
            .get_mut(&key)
        {
            state.in_flight = state.in_flight.saturating_sub(1);
        }
    }
}
