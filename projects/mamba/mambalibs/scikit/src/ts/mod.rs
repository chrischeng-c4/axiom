//! Time series analysis module.
//!
//! - **arima**: ARIMA(p,d,q) modeling and forecasting
//! - **decompose**: Seasonal decomposition (additive/multiplicative)
//! - **smoothing**: Exponential smoothing (SES, Holt, Holt-Winters)
//! - **acf**: Autocorrelation, partial autocorrelation, Ljung-Box test
//! - **resample**: Downsample/upsample with aggregation and interpolation
//! - **ops**: EWMA, expanding window, lag/lead helpers

mod error;

pub mod acf;
pub mod arima;
pub mod decompose;
pub mod ops;
pub mod resample;
pub mod smoothing;

pub use acf::{acf, ljung_box, pacf, LjungBoxResult};
pub use arima::{ArimaModel, ArimaOrder};
pub use decompose::{seasonal_decompose, DecomposeModel, DecomposeResult};
pub use error::{Result, TsError};
pub use ops::SeriesTimeSeriesExt;
pub use ops::{ewma, ewma_halflife, ewma_span, expanding_max, expanding_mean};
pub use ops::{expanding_min, expanding_std, expanding_sum, lag, lead};
pub use resample::{downsample, resample_to_len, upsample, AggMethod, InterpMethod};
pub use smoothing::{
    holt, holt_winters, ses, HoltResult, HoltWintersParams, HoltWintersResult, HwSeasonal,
    SesResult,
};
