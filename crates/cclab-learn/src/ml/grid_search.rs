//! Grid search with cross-validation for hyperparameter tuning.

use std::collections::HashMap;

use super::cross_validation::cross_val_score;
use super::error::{MlError, Result};
use super::traits::{Estimator, Predictor};

/// Result of a grid search cross-validation.
#[derive(Debug, Clone)]
pub struct GridSearchResult {
    /// Best mean CV score across all parameter combinations.
    pub best_score: f64,
    /// Parameter combination that achieved the best score.
    pub best_params: HashMap<String, f64>,
    /// All tested parameter combinations with their mean CV scores.
    pub cv_results: Vec<(HashMap<String, f64>, f64)>,
}

/// Perform exhaustive grid search over parameter combinations with cross-validation.
///
/// # Arguments
/// * `build_model` - Factory function: given a parameter map, returns a model.
/// * `param_grid` - Map of parameter names to lists of values to try.
/// * `x` - Flat feature matrix (n_samples * n_features), row-major.
/// * `n_features` - Number of features per sample.
/// * `y` - Target vector (n_samples).
/// * `cv` - Number of cross-validation folds.
/// * `scoring` - Scoring function: `"accuracy"`, `"mse"`, etc.
///
/// # Returns
/// A `GridSearchResult` with the best parameters and all CV results.
pub fn grid_search_cv<F, E>(
    build_model: F,
    param_grid: &HashMap<String, Vec<f64>>,
    x: &[f64],
    n_features: usize,
    y: &[f64],
    cv: usize,
    scoring: &str,
) -> Result<GridSearchResult>
where
    F: Fn(&HashMap<String, f64>) -> E,
    E: Clone + Estimator + Predictor,
{
    let combos = enumerate_param_combos(param_grid);
    if combos.is_empty() {
        return Err(MlError::InvalidParameter(
            "param_grid produced no combinations".into(),
        ));
    }

    let mut cv_results: Vec<(HashMap<String, f64>, f64)> = Vec::with_capacity(combos.len());
    let mut best_score = f64::NEG_INFINITY;
    let mut best_params = combos[0].clone();

    for params in &combos {
        let model = build_model(params);
        let fold_scores = cross_val_score(&model, x, n_features, y, cv, scoring)?;
        let mean_score = fold_scores.iter().sum::<f64>() / fold_scores.len() as f64;

        cv_results.push((params.clone(), mean_score));

        if mean_score > best_score {
            best_score = mean_score;
            best_params = params.clone();
        }
    }

    Ok(GridSearchResult {
        best_score,
        best_params,
        cv_results,
    })
}

/// Enumerate all combinations from the parameter grid (cartesian product).
fn enumerate_param_combos(grid: &HashMap<String, Vec<f64>>) -> Vec<HashMap<String, f64>> {
    let keys: Vec<&String> = grid.keys().collect();
    if keys.is_empty() {
        return vec![HashMap::new()];
    }

    let mut combos = vec![HashMap::new()];

    for key in &keys {
        let values = &grid[*key];
        let mut new_combos = Vec::with_capacity(combos.len() * values.len());
        for combo in &combos {
            for &val in values {
                let mut new_combo = combo.clone();
                new_combo.insert((*key).clone(), val);
                new_combos.push(new_combo);
            }
        }
        combos = new_combos;
    }

    combos
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ml::knn::{KNeighborsClassifier, KNeighborsRegressor};
    use crate::ml::linear::RidgeRegression;

    #[test]
    fn test_grid_search_ridge_alpha() {
        // y = 2*x + 1
        let x: Vec<f64> = (1..=20).map(|i| i as f64).collect();
        let y: Vec<f64> = (1..=20).map(|i| (i * 2 + 1) as f64).collect();

        let mut param_grid = HashMap::new();
        param_grid.insert("alpha".to_string(), vec![0.01, 0.1, 1.0, 10.0]);

        let result = grid_search_cv(
            |params| {
                let alpha = params["alpha"];
                RidgeRegression::new(alpha)
            },
            &param_grid,
            &x,
            1,
            &y,
            5,
            "mse",
        )
        .unwrap();

        // Should have tested 4 parameter combos
        assert_eq!(result.cv_results.len(), 4);
        // Best alpha should be one of the small values (less regularization
        // is better for a perfectly linear relationship)
        assert!(
            result.best_params["alpha"] <= 1.0,
            "expected small alpha, got {}",
            result.best_params["alpha"]
        );
        // Best score is neg_mse, should be close to 0 (less negative is better)
        assert!(
            result.best_score > -5.0,
            "expected good MSE, got {}",
            result.best_score
        );
    }

    #[test]
    fn test_grid_search_knn_k() {
        // Classification: two well-separated clusters, interleaved so
        // each CV fold gets both classes in the training set.
        let x = vec![
            0.0, 0.0, //  class 0
            10.0, 10.0, // class 1
            0.1, 0.1, //  class 0
            10.1, 10.1, // class 1
            0.2, 0.2, //  class 0
            10.2, 10.2, // class 1
            0.3, 0.3, //  class 0
            10.3, 10.3, // class 1
        ];
        let y = vec![0.0, 1.0, 0.0, 1.0, 0.0, 1.0, 0.0, 1.0];

        let mut param_grid = HashMap::new();
        param_grid.insert("k".to_string(), vec![1.0, 3.0]);

        let result = grid_search_cv(
            |params| {
                let k = params["k"] as usize;
                KNeighborsClassifier::new(k)
            },
            &param_grid,
            &x,
            2,
            &y,
            2,
            "accuracy",
        )
        .unwrap();

        assert_eq!(result.cv_results.len(), 2);
        // Both k=1 and k=3 should achieve good accuracy on well-separated data
        assert!(
            result.best_score >= 0.75,
            "expected high accuracy, got {}",
            result.best_score
        );
    }

    #[test]
    fn test_grid_search_multi_param() {
        // Test with 2 parameters to verify cartesian product
        // Use KNeighborsRegressor: only k matters, but we add a dummy param
        let x: Vec<f64> = (1..=12).map(|i| i as f64).collect();
        let y: Vec<f64> = (1..=12).map(|i| (i * 2) as f64).collect();

        let mut param_grid = HashMap::new();
        param_grid.insert("k".to_string(), vec![1.0, 2.0, 3.0]);
        param_grid.insert("dummy".to_string(), vec![0.0, 1.0]);

        let result = grid_search_cv(
            |params| {
                let k = params["k"] as usize;
                KNeighborsRegressor::new(k)
            },
            &param_grid,
            &x,
            1,
            &y,
            3,
            "mse",
        )
        .unwrap();

        // 3 k values * 2 dummy values = 6 combos
        assert_eq!(result.cv_results.len(), 6);
    }

    #[test]
    fn test_grid_search_empty_grid() {
        let x = vec![1.0, 2.0, 3.0, 4.0];
        let y = vec![1.0, 2.0, 3.0, 4.0];
        let param_grid: HashMap<String, Vec<f64>> = HashMap::new();

        // Empty grid produces one combo (default params)
        let result = grid_search_cv(
            |_params| RidgeRegression::new(0.1),
            &param_grid,
            &x,
            1,
            &y,
            2,
            "mse",
        )
        .unwrap();

        assert_eq!(result.cv_results.len(), 1);
    }

    #[test]
    fn test_enumerate_param_combos() {
        let mut grid = HashMap::new();
        grid.insert("a".to_string(), vec![1.0, 2.0]);
        grid.insert("b".to_string(), vec![10.0, 20.0, 30.0]);
        let combos = enumerate_param_combos(&grid);
        assert_eq!(combos.len(), 6); // 2 * 3
                                     // Each combo should have both keys
        for combo in &combos {
            assert!(combo.contains_key("a"));
            assert!(combo.contains_key("b"));
        }
    }
}
