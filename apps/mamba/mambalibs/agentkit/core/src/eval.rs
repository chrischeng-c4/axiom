//! Eval trait + dataset runner (#2060).
//!
//! Deterministic testing harness for any async "system under test"
//! shaped like `Input -> Output`. A typical use is wiring an
//! [`Agent`](crate::Agent) (or a `Step`/`Graph`) behind the [`Eval`]
//! trait, then running it across a labelled [`Dataset`]; the runner
//! captures each case's actual output, scores it against the
//! ground-truth label, and produces a typed [`EvalReport`].
//!
//! Scoring is pluggable through the [`Scorer`] trait. Two built-in
//! scorers ship in this slice:
//!
//! * [`ExactMatchScorer`] — `==` between expected and actual.
//! * [`JsonEqScorer`]    — `Output: Serialize` round-tripped through
//!                          `serde_json::Value` then compared, useful
//!                          for typed structured outputs where field
//!                          order shouldn't matter.

// HANDWRITE-BEGIN reason: no rust-runtime generator for an eval
// harness yet. Same Epic-3 gap as the rest of agentkit-core.

use std::marker::PhantomData;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::error::{NovaError, NovaResult};

/// System under evaluation. Implementors are typed to a single
/// `Input`/`Output` shape so the runner can drive them deterministically.
#[async_trait]
pub trait Eval<Input, Output>: Send + Sync
where
    Input: Send + 'static,
    Output: Send + 'static,
{
    /// Drive the system once with `input`. The runner captures the
    /// result (Ok or Err) and records it on the report — a single case
    /// failing does not abort the whole run.
    async fn run(&self, input: Input) -> NovaResult<Output>;
}

/// A single labelled input/expected pair.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EvalCase<Input, Output> {
    /// Stable identifier — used for filtering, retries, and report
    /// rows. Caller's responsibility to keep unique within a dataset.
    pub id: String,
    pub input: Input,
    pub expected: Output,
}

impl<Input, Output> EvalCase<Input, Output> {
    pub fn new(id: impl Into<String>, input: Input, expected: Output) -> Self {
        Self {
            id: id.into(),
            input,
            expected,
        }
    }
}

/// Labelled collection of [`EvalCase`]s. Pure data — the runner does
/// the work.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Dataset<Input, Output> {
    pub name: String,
    pub cases: Vec<EvalCase<Input, Output>>,
}

impl<Input, Output> Dataset<Input, Output> {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            cases: Vec::new(),
        }
    }

    pub fn with_case(mut self, case: EvalCase<Input, Output>) -> Self {
        self.cases.push(case);
        self
    }

    pub fn push(&mut self, case: EvalCase<Input, Output>) {
        self.cases.push(case);
    }

    pub fn len(&self) -> usize {
        self.cases.len()
    }

    pub fn is_empty(&self) -> bool {
        self.cases.is_empty()
    }
}

/// Normalised score in `0.0..=1.0` plus an optional human-readable
/// reason. `1.0` is a perfect match; `0.0` is a complete miss.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Score {
    pub value: f32,
    pub reason: Option<String>,
}

impl Score {
    pub fn pass() -> Self {
        Self {
            value: 1.0,
            reason: None,
        }
    }
    pub fn fail(reason: impl Into<String>) -> Self {
        Self {
            value: 0.0,
            reason: Some(reason.into()),
        }
    }
    pub fn partial(value: f32, reason: impl Into<String>) -> Self {
        Self {
            value: value.clamp(0.0, 1.0),
            reason: Some(reason.into()),
        }
    }
}

/// Strategy for grading a single case's output against its expected
/// label.
pub trait Scorer<Output>: Send + Sync {
    fn score(&self, expected: &Output, actual: &Output) -> Score;
}

/// Default scorer: `==`. Requires `Output: PartialEq`.
pub struct ExactMatchScorer<Output> {
    _marker: PhantomData<fn() -> Output>,
}

impl<Output> ExactMatchScorer<Output> {
    pub fn new() -> Self {
        Self {
            _marker: PhantomData,
        }
    }
}

impl<Output> Default for ExactMatchScorer<Output> {
    fn default() -> Self {
        Self::new()
    }
}

impl<Output> Scorer<Output> for ExactMatchScorer<Output>
where
    Output: PartialEq + std::fmt::Debug + Send + Sync,
{
    fn score(&self, expected: &Output, actual: &Output) -> Score {
        if expected == actual {
            Score::pass()
        } else {
            Score::fail(format!("expected {expected:?}, got {actual:?}"))
        }
    }
}

/// Scorer for structured outputs: `serde_json::Value` equality after
/// a round-trip. Field order is therefore irrelevant.
pub struct JsonEqScorer<Output> {
    _marker: PhantomData<fn() -> Output>,
}

impl<Output> JsonEqScorer<Output> {
    pub fn new() -> Self {
        Self {
            _marker: PhantomData,
        }
    }
}

impl<Output> Default for JsonEqScorer<Output> {
    fn default() -> Self {
        Self::new()
    }
}

impl<Output> Scorer<Output> for JsonEqScorer<Output>
where
    Output: Serialize + Send + Sync,
{
    fn score(&self, expected: &Output, actual: &Output) -> Score {
        let e = match serde_json::to_value(expected) {
            Ok(v) => v,
            Err(e) => return Score::fail(format!("serialize expected: {e}")),
        };
        let a = match serde_json::to_value(actual) {
            Ok(v) => v,
            Err(e) => return Score::fail(format!("serialize actual: {e}")),
        };
        if e == a {
            Score::pass()
        } else {
            Score::fail(format!("json mismatch: expected={e}, actual={a}"))
        }
    }
}

/// One row in the [`EvalReport`].
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EvalCaseResult {
    pub id: String,
    pub score: Score,
    /// Error message when the system under test returned `Err`. When
    /// set, `score` is forced to `0.0` with a stock reason.
    pub error: Option<String>,
}

/// Aggregate result of running a [`Dataset`] through an [`Eval`].
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EvalReport {
    pub dataset: String,
    pub cases: Vec<EvalCaseResult>,
}

impl EvalReport {
    pub fn pass_rate(&self) -> f32 {
        if self.cases.is_empty() {
            return 0.0;
        }
        let total: f32 = self.cases.iter().map(|c| c.score.value).sum();
        total / self.cases.len() as f32
    }

    pub fn failures(&self) -> impl Iterator<Item = &EvalCaseResult> {
        self.cases.iter().filter(|c| c.score.value < 1.0)
    }
}

/// Driver that runs an [`Eval`] across a [`Dataset`] and scores each
/// case with the provided [`Scorer`]. Cases run sequentially in this
/// slice; parallel fan-out is a future extension once cancellation
/// (#2070) lands.
pub struct DatasetRunner<S> {
    scorer: S,
}

impl<S> DatasetRunner<S> {
    pub fn new(scorer: S) -> Self {
        Self { scorer }
    }
}

impl<S> DatasetRunner<S> {
    pub async fn run<E, Input, Output>(
        &self,
        eval: &E,
        dataset: &Dataset<Input, Output>,
    ) -> NovaResult<EvalReport>
    where
        E: Eval<Input, Output> + ?Sized,
        S: Scorer<Output>,
        Input: Clone + Send + 'static,
        Output: Send + 'static,
    {
        let mut cases = Vec::with_capacity(dataset.cases.len());
        for case in &dataset.cases {
            match eval.run(case.input.clone()).await {
                Ok(actual) => {
                    let score = self.scorer.score(&case.expected, &actual);
                    cases.push(EvalCaseResult {
                        id: case.id.clone(),
                        score,
                        error: None,
                    });
                }
                Err(e) => {
                    cases.push(EvalCaseResult {
                        id: case.id.clone(),
                        score: Score::fail("eval returned Err"),
                        error: Some(e.to_string()),
                    });
                }
            }
        }
        if cases.is_empty() {
            return Err(NovaError::ConfigError(
                "dataset has no cases — nothing to evaluate".into(),
            ));
        }
        Ok(EvalReport {
            dataset: dataset.name.clone(),
            cases,
        })
    }
}

// HANDWRITE-END

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone)]
    struct Echo;

    #[async_trait]
    impl Eval<String, String> for Echo {
        async fn run(&self, input: String) -> NovaResult<String> {
            Ok(input)
        }
    }

    #[tokio::test]
    async fn happy_path_all_cases_match() {
        let ds: Dataset<String, String> = Dataset::new("echo")
            .with_case(EvalCase::new("a", "hello".into(), "hello".into()))
            .with_case(EvalCase::new("b", "world".into(), "world".into()));

        let runner = DatasetRunner::new(ExactMatchScorer::<String>::new());
        let report = runner.run(&Echo, &ds).await.unwrap();

        assert_eq!(report.dataset, "echo");
        assert_eq!(report.cases.len(), 2);
        assert_eq!(report.pass_rate(), 1.0);
        assert!(report.failures().next().is_none());
    }

    #[tokio::test]
    async fn mismatch_is_recorded_as_zero_score_with_reason() {
        let ds: Dataset<String, String> = Dataset::new("echo")
            .with_case(EvalCase::new("ok", "x".into(), "x".into()))
            .with_case(EvalCase::new("bad", "x".into(), "y".into()));

        let runner = DatasetRunner::new(ExactMatchScorer::<String>::new());
        let report = runner.run(&Echo, &ds).await.unwrap();

        assert_eq!(report.pass_rate(), 0.5);
        let failures: Vec<_> = report.failures().collect();
        assert_eq!(failures.len(), 1);
        assert_eq!(failures[0].id, "bad");
        assert!(failures[0].score.reason.as_ref().unwrap().contains("\"x\""));
    }

    #[tokio::test]
    async fn err_from_eval_is_captured_per_case() {
        #[derive(Clone)]
        struct Boom;
        #[async_trait]
        impl Eval<u32, u32> for Boom {
            async fn run(&self, _: u32) -> NovaResult<u32> {
                Err(NovaError::LLMError("upstream is down".into()))
            }
        }
        let ds: Dataset<u32, u32> = Dataset::new("boom").with_case(EvalCase::new("only", 1, 1));
        let runner = DatasetRunner::new(ExactMatchScorer::<u32>::new());
        let report = runner.run(&Boom, &ds).await.unwrap();

        assert_eq!(report.cases.len(), 1);
        assert_eq!(report.cases[0].score.value, 0.0);
        assert_eq!(
            report.cases[0].error.as_deref().unwrap(),
            "LLM provider error: upstream is down"
        );
    }

    #[tokio::test]
    async fn empty_dataset_returns_typed_error() {
        let ds: Dataset<String, String> = Dataset::new("empty");
        let runner = DatasetRunner::new(ExactMatchScorer::<String>::new());
        let err = runner.run(&Echo, &ds).await.unwrap_err();
        assert!(matches!(err, NovaError::ConfigError(m) if m.contains("no cases")));
    }

    #[tokio::test]
    async fn json_eq_scorer_ignores_field_order() {
        #[derive(Clone, Serialize)]
        struct Out {
            a: u32,
            b: u32,
        }
        #[derive(Clone)]
        struct Mk;
        #[async_trait]
        impl Eval<(), Out> for Mk {
            async fn run(&self, _: ()) -> NovaResult<Out> {
                Ok(Out { a: 1, b: 2 })
            }
        }
        let ds: Dataset<(), Out> =
            Dataset::new("json").with_case(EvalCase::new("c1", (), Out { a: 1, b: 2 }));
        let runner = DatasetRunner::new(JsonEqScorer::<Out>::new());
        let report = runner.run(&Mk, &ds).await.unwrap();
        assert_eq!(report.pass_rate(), 1.0);
    }

    #[test]
    fn score_clamps_partial_into_unit_interval() {
        assert_eq!(Score::partial(-0.5, "x").value, 0.0);
        assert_eq!(Score::partial(1.5, "x").value, 1.0);
        assert_eq!(Score::partial(0.42, "x").value, 0.42);
    }
}
