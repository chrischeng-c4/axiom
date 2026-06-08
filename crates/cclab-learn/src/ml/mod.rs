//! Machine learning algorithms module.
//!
//! - **linear**: OLS, Ridge, Lasso regression
//! - **logistic**: Logistic regression
//! - **tree**: Decision tree (CART)
//! - **cluster**: K-means clustering
//! - **pca**: Principal Component Analysis
//! - **preprocessing**: StandardScaler, MinMaxScaler, LabelEncoder, OneHotEncoder
//! - **split**: train_test_split
//! - **metrics**: accuracy, precision, recall, F1, MSE, R², etc.
//! - **knn**: K-Nearest Neighbors
//! - **svm**: Linear SVM
//! - **naive_bayes**: Gaussian Naive Bayes
//! - **ensemble**: Random Forest, Gradient Boosting
//! - **dbscan**: DBSCAN clustering
//! - **pipeline**: ML Pipeline
//! - **cross_validation**: K-fold cross-validation scoring
//! - **grid_search**: Grid search with cross-validation
//! - **feature_eng**: PolynomialFeatures, CountVectorizer

mod error;
pub mod traits;

pub mod cluster;
pub mod cross_validation;
pub mod dbscan;
pub mod ensemble;
pub mod feature_eng;
pub mod grid_search;
pub mod knn;
pub mod linear;
pub mod logistic;
pub mod metrics;
pub mod naive_bayes;
pub mod pca;
pub mod pipeline;
pub mod preprocessing;
pub mod split;
pub mod svm;
pub mod tree;

pub use cluster::KMeans;
pub use cross_validation::cross_val_score;
pub use dbscan::DBSCAN;
pub use ensemble::{GradientBoostingClassifier, RandomForestClassifier};
pub use error::{MlError, Result};
pub use feature_eng::{CountVectorizer, PolynomialFeatures};
pub use grid_search::{grid_search_cv, GridSearchResult};
pub use knn::{KNeighborsClassifier, KNeighborsRegressor};
pub use linear::{LassoRegression, LinearRegression, RidgeRegression};
pub use logistic::LogisticRegression;
pub use naive_bayes::GaussianNB;
pub use pca::PCA;
pub use pipeline::Pipeline;
pub use preprocessing::{LabelEncoder, MinMaxScaler, OneHotEncoder, StandardScaler};
pub use split::{train_test_split, SplitResult};
pub use svm::LinearSVC;
pub use traits::{Classifier, Estimator, Predictor, Transformer};
pub use tree::{Criterion, DecisionTree};
