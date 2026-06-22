use super::super::rc::{MbObject, ObjData};
use super::super::value::MbValue;
/// cmath module for Mamba (#1265 Task #38, wave-2 ship #1).
///
/// Complex math functions backed by libstd f64. Complex values are
/// `ObjData::Complex(re, im)` (so `.real` / `.imag` attribute access
/// works through class.rs::attribute_lookup). Module-level only —
/// no classes. Surface: 24 def + 7 constants = 31.
///
/// HANDWRITE-BEGIN reason: stdlib-shim section type (register_module +
/// flat-args dispatch + tuple-table + Complex variant) is not yet
/// emitted by score codegen. Same shape as codecs_mod / math_mod —
/// handwrite during brute-force Phase 2, replace when score
/// standardize lands the stdlib-shim section type.
use std::collections::HashMap;

// ── Variadic dispatchers (callable from module-attr context) ──
// NOTE: dispatcher fn names must start with `dispatch_` so the surface
// walker (projects/mamba/src/surface.rs::pick_tuple_dispatcher) recognises
// them. Without the prefix Gate 3 surface scores 0/N.

macro_rules! disp_unary {
    ($disp:ident, $fn:path) => {
        unsafe extern "C" fn $disp(args_ptr: *const MbValue, nargs: usize) -> MbValue {
            let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
            $fn(a.first().copied().unwrap_or_else(MbValue::none))
        }
    };
}

macro_rules! disp_binary {
    ($disp:ident, $fn:path) => {
        unsafe extern "C" fn $disp(args_ptr: *const MbValue, nargs: usize) -> MbValue {
            let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
            $fn(
                a.first().copied().unwrap_or_else(MbValue::none),
                a.get(1).copied().unwrap_or_else(MbValue::none),
            )
        }
    };
}

// Trigonometric
disp_unary!(dispatch_acos, mb_cmath_acos);
disp_unary!(dispatch_asin, mb_cmath_asin);
disp_unary!(dispatch_atan, mb_cmath_atan);
disp_unary!(dispatch_cos, mb_cmath_cos);
disp_unary!(dispatch_sin, mb_cmath_sin);
disp_unary!(dispatch_tan, mb_cmath_tan);

// Hyperbolic
disp_unary!(dispatch_acosh, mb_cmath_acosh);
disp_unary!(dispatch_asinh, mb_cmath_asinh);
disp_unary!(dispatch_atanh, mb_cmath_atanh);
disp_unary!(dispatch_cosh, mb_cmath_cosh);
disp_unary!(dispatch_sinh, mb_cmath_sinh);
disp_unary!(dispatch_tanh, mb_cmath_tanh);

// Exp / log / power
disp_unary!(dispatch_exp, mb_cmath_exp);
disp_unary!(dispatch_log10, mb_cmath_log10);
disp_unary!(dispatch_sqrt, mb_cmath_sqrt);

// Predicates
disp_unary!(dispatch_isfinite, mb_cmath_isfinite);
disp_unary!(dispatch_isinf, mb_cmath_isinf);
disp_unary!(dispatch_isnan, mb_cmath_isnan);

// Coordinate conversion
disp_unary!(dispatch_phase, mb_cmath_phase);
disp_unary!(dispatch_polar, mb_cmath_polar);
disp_binary!(dispatch_rect, mb_cmath_rect);

// Binary-arg: log(z, base=e). 1-or-2 positional; missing base → natural log.
disp_binary!(dispatch_log, mb_cmath_log);

// isclose(a, b, *, rel_tol=1e-09, abs_tol=0.0). CPython treats rel_tol /
// abs_tol as keyword-only; mamba passes positionally so accept up to 4
// positional args and apply defaults for missing ones.
unsafe extern "C" fn dispatch_isclose(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    // rel_tol / abs_tol are keyword-only: read the trailing kwargs dict.
    let kw = a.last().copied().filter(|v| {
        v.as_ptr()
            .is_some_and(|p| unsafe { matches!((*p).data, ObjData::Dict(_)) })
    });
    let kwarg = |key: &str| -> Option<MbValue> {
        let ptr = kw?.as_ptr()?;
        unsafe {
            if let ObjData::Dict(ref lock) = (*ptr).data {
                return lock.read().unwrap().get(key).copied();
            }
        }
        None
    };
    let z1 = a.first().copied().unwrap_or_else(MbValue::none);
    let z2 = a.get(1).copied().unwrap_or_else(MbValue::none);
    let mut rel_tol = 1e-9;
    let mut abs_tol = 0.0;
    for (name, slot, default_pos) in [
        ("rel_tol", &mut rel_tol as *mut f64, 2usize),
        ("abs_tol", &mut abs_tol as *mut f64, 3usize),
    ] {
        let v = kwarg(name).or_else(|| {
            a.get(default_pos).copied().filter(|x| {
                !x.as_ptr()
                    .is_some_and(|p| unsafe { matches!((*p).data, ObjData::Dict(_)) })
            })
        });
        if let Some(v) = v {
            // A complex tolerance is a TypeError (must be real).
            let is_complex = v
                .as_ptr()
                .is_some_and(|p| unsafe { matches!((*p).data, ObjData::Complex(..)) });
            if is_complex {
                super::super::exception::mb_raise(
                    MbValue::from_ptr(MbObject::new_str("TypeError".to_string())),
                    MbValue::from_ptr(MbObject::new_str(format!(
                        "can't convert complex to float ({name})"
                    ))),
                );
                return MbValue::none();
            }
            let f = as_f64(v).unwrap_or(0.0);
            if f < 0.0 {
                super::super::exception::mb_raise(
                    MbValue::from_ptr(MbObject::new_str("ValueError".to_string())),
                    MbValue::from_ptr(MbObject::new_str(
                        "tolerances must be non-negative".to_string(),
                    )),
                );
                return MbValue::none();
            }
            unsafe { *slot = f };
        }
    }
    mb_cmath_isclose(z1, z2, rel_tol, abs_tol)
}

/// Register the cmath module — 24 functions + 7 constants.
pub fn register() {
    let mut attrs = HashMap::new();

    let dispatchers: Vec<(&str, usize)> = vec![
        ("acos", dispatch_acos as *const () as usize),
        ("acosh", dispatch_acosh as *const () as usize),
        ("asin", dispatch_asin as *const () as usize),
        ("asinh", dispatch_asinh as *const () as usize),
        ("atan", dispatch_atan as *const () as usize),
        ("atanh", dispatch_atanh as *const () as usize),
        ("cos", dispatch_cos as *const () as usize),
        ("cosh", dispatch_cosh as *const () as usize),
        ("exp", dispatch_exp as *const () as usize),
        ("isclose", dispatch_isclose as *const () as usize),
        ("isfinite", dispatch_isfinite as *const () as usize),
        ("isinf", dispatch_isinf as *const () as usize),
        ("isnan", dispatch_isnan as *const () as usize),
        ("log", dispatch_log as *const () as usize),
        ("log10", dispatch_log10 as *const () as usize),
        ("phase", dispatch_phase as *const () as usize),
        ("polar", dispatch_polar as *const () as usize),
        ("rect", dispatch_rect as *const () as usize),
        ("sin", dispatch_sin as *const () as usize),
        ("sinh", dispatch_sinh as *const () as usize),
        ("sqrt", dispatch_sqrt as *const () as usize),
        ("tan", dispatch_tan as *const () as usize),
        ("tanh", dispatch_tanh as *const () as usize),
    ];
    for (name, addr) in dispatchers {
        attrs.insert(name.to_string(), MbValue::from_func(addr));
        super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
            s.borrow_mut().insert(addr as u64);
        });
    }

    // Constants
    attrs.insert("pi".to_string(), MbValue::from_float(std::f64::consts::PI));
    attrs.insert("e".to_string(), MbValue::from_float(std::f64::consts::E));
    attrs.insert(
        "tau".to_string(),
        MbValue::from_float(std::f64::consts::TAU),
    );
    attrs.insert("inf".to_string(), MbValue::from_float(f64::INFINITY));
    attrs.insert("infj".to_string(), make_complex(0.0, f64::INFINITY));
    attrs.insert("nan".to_string(), MbValue::from_float(f64::NAN));
    attrs.insert("nanj".to_string(), make_complex(0.0, f64::NAN));

    super::register_module("cmath", attrs);
}

// ── Helpers ──

fn as_f64(val: MbValue) -> Option<f64> {
    val.as_float().or_else(|| val.as_int().map(|i| i as f64))
}

/// Extract (real, imag) from a Complex, 2-tuple, real float, or int.
fn extract_complex(val: MbValue) -> (f64, f64) {
    if let Some(f) = val.as_float() {
        return (f, 0.0);
    }
    if let Some(i) = val.as_int() {
        return (i as f64, 0.0);
    }
    if let Some(ptr) = val.as_ptr() {
        unsafe {
            match (*ptr).data {
                ObjData::Complex(re, im) => return (re, im),
                ObjData::Tuple(ref items) if items.len() >= 2 => {
                    let r = as_f64(items[0]).unwrap_or(0.0);
                    let i = as_f64(items[1]).unwrap_or(0.0);
                    return (r, i);
                }
                _ => {}
            }
        }
    }
    (0.0, 0.0)
}

fn make_complex(real: f64, imag: f64) -> MbValue {
    MbValue::from_ptr(MbObject::new_complex(real, imag))
}

// ── Internal: complex math, faithful port of CPython 3.12 cmathmodule.c ──
//
// Reproduces CPython's specialised numerically-stable formulas, signed-zero
// conventions, special-value (inf/nan) tables, and domain/overflow errors so
// that results match byte-for-byte. Each public function records a libm-style
// `errno` (EDOM → ValueError, ERANGE → OverflowError) which the dispatcher
// surface translates into a raised exception.

#[derive(Clone, Copy)]
struct Errno {
    edom: bool,   // domain error → ValueError
    erange: bool, // range error → OverflowError
}
impl Errno {
    fn new() -> Self {
        Errno {
            edom: false,
            erange: false,
        }
    }
}

const CM_LARGE_DOUBLE: f64 = f64::MAX / 4.0;
const M_LN2: f64 = std::f64::consts::LN_2; // 0.6931471805599453
const M_LN10: f64 = std::f64::consts::LN_10; // 2.302585092994046

// CM_SCALE_UP must be even; CM_SCALE_DOWN = -(CM_SCALE_UP+1)/2.
// CPython uses 2*((DBL_MIN_EXP - DBL_MANT_DIG)/2) = 2*((-1021 - 53)/2) = -1074.
const CM_SCALE_UP: i32 = 2 * ((-1021 - 53) / 2);
const CM_SCALE_DOWN: i32 = -(CM_SCALE_UP + 1) / 2;

// Used only by the unit tests below; kept for test-side complex arithmetic.
#[allow(dead_code)]
fn cadd(a: (f64, f64), b: (f64, f64)) -> (f64, f64) {
    (a.0 + b.0, a.1 + b.1)
}
#[allow(dead_code)]
fn csub(a: (f64, f64), b: (f64, f64)) -> (f64, f64) {
    (a.0 - b.0, a.1 - b.1)
}
#[allow(dead_code)]
fn cmul(a: (f64, f64), b: (f64, f64)) -> (f64, f64) {
    (a.0 * b.0 - a.1 * b.1, a.0 * b.1 + a.1 * b.0)
}

fn ldexp(x: f64, exp: i32) -> f64 {
    // libc ldexp: x * 2^exp, preserving inf/nan/zero like CPython.
    if x == 0.0 || !x.is_finite() {
        return x;
    }
    x * exp2i(exp)
}
fn exp2i(exp: i32) -> f64 {
    // 2^exp built up to avoid intermediate over/underflow for large |exp|.
    if exp >= 0 {
        let mut r = 1.0_f64;
        let mut e = exp;
        while e > 1023 {
            r *= 2.0_f64.powi(1023);
            e -= 1023;
        }
        r * 2.0_f64.powi(e)
    } else {
        let mut r = 1.0_f64;
        let mut e = exp;
        while e < -1022 {
            r *= 2.0_f64.powi(-1022);
            e += 1022;
        }
        r * 2.0_f64.powi(e)
    }
}

fn copysign(mag: f64, sign: f64) -> f64 {
    mag.copysign(sign)
}

// log1p with CPython's accuracy fallback. Rust's ln_1p matches C log1p.
fn log1p(x: f64) -> f64 {
    x.ln_1p()
}

// Accurate real asinh matching the system libm CPython links against.
// Rust's intrinsic f64::asinh can differ by a couple of ULP from glibc/Apple
// libm; reproduce the standard high-accuracy reduction so complex inverse
// functions agree with CPython bit-for-bit.
fn m_asinh(x: f64) -> f64 {
    if !x.is_finite() {
        return x;
    }
    let ax = x.abs();
    if ax < 1e-9 {
        // asinh(x) ≈ x for tiny arguments (avoids spurious underflow work).
        return x;
    }
    let w = if ax > 1.0e18 {
        // Large: asinh(x) = ln(2|x|) = ln(|x|) + ln(2).
        ax.ln() + M_LN2
    } else if ax > 2.0 {
        (2.0 * ax + 1.0 / (ax + (ax * ax + 1.0).sqrt())).ln()
    } else {
        let t = ax * ax;
        log1p(ax + t / (1.0 + (1.0 + t).sqrt()))
    };
    copysign(w, x)
}

// ── special-value tables ─────────────────────────────────────────────────────
// Classify a float into one of 7 bins matching CPython's SPECIAL_VALUE macro:
// 0:-inf 1:-finite 2:-0 3:+0 4:+finite 5:+inf 6:nan.
fn special_type(d: f64) -> usize {
    if d.is_finite() {
        if d != 0.0 {
            if d.is_sign_negative() {
                1
            } else {
                4
            }
        } else if d.is_sign_negative() {
            2
        } else {
            3
        }
    } else if d.is_nan() {
        6
    } else if d.is_sign_negative() {
        0
    } else {
        5
    }
}

const P: f64 = std::f64::consts::PI;
const P14: f64 = std::f64::consts::FRAC_PI_4;
const P12: f64 = std::f64::consts::FRAC_PI_2;
const P34: f64 = 3.0 * std::f64::consts::FRAC_PI_4;
const INF: f64 = f64::INFINITY;
const N: f64 = f64::NAN;
const U: f64 = f64::NAN; // "never used" special-value table slot sentinel

fn special_value(table: &[[(f64, f64); 7]; 7], x: f64, y: f64) -> (f64, f64) {
    table[special_type(x)][special_type(y)]
}

// ── c_abs ────────────────────────────────────────────────────────────────────
fn cabs(z: (f64, f64)) -> f64 {
    // CPython _Py_c_abs: hypot with inf/nan special-casing. C99 hypot returns
    // +inf if either part is inf, even if the other is nan.
    if !z.0.is_finite() || !z.1.is_finite() {
        if z.0.is_infinite() {
            return z.0.abs();
        }
        if z.1.is_infinite() {
            return z.1.abs();
        }
        return f64::NAN;
    }
    z.0.hypot(z.1)
}

// ── sqrt ───────────────────────────────────────────────────────────────────
fn csqrt(z: (f64, f64), _errno: &mut Errno) -> (f64, f64) {
    let (x, y) = z;
    if !x.is_finite() || !y.is_finite() {
        const SQRT_SPECIAL: [[(f64, f64); 7]; 7] = [
            [
                (INF, -INF),
                (0.0, -INF),
                (0.0, -INF),
                (0.0, INF),
                (0.0, INF),
                (INF, INF),
                (INF, N),
            ],
            [
                (INF, -INF),
                (U, U),
                (U, U),
                (U, U),
                (U, U),
                (INF, INF),
                (N, N),
            ],
            [
                (INF, -INF),
                (U, U),
                (0.0, -0.0),
                (0.0, 0.0),
                (U, U),
                (INF, INF),
                (N, N),
            ],
            [
                (INF, -INF),
                (U, U),
                (0.0, -0.0),
                (0.0, 0.0),
                (U, U),
                (INF, INF),
                (N, N),
            ],
            [
                (INF, -INF),
                (U, U),
                (U, U),
                (U, U),
                (U, U),
                (INF, INF),
                (N, N),
            ],
            [
                (INF, -INF),
                (INF, -0.0),
                (INF, -0.0),
                (INF, 0.0),
                (INF, 0.0),
                (INF, INF),
                (INF, N),
            ],
            [
                (INF, -INF),
                (N, N),
                (N, N),
                (N, N),
                (N, N),
                (INF, INF),
                (N, N),
            ],
        ];
        return special_value(&SQRT_SPECIAL, x, y);
    }
    if x == 0.0 && y == 0.0 {
        return (0.0, copysign(0.0, y));
    }

    let mut ax = x.abs();
    let ay = y.abs();

    let s;
    if ax < f64::MIN_POSITIVE && ay < f64::MIN_POSITIVE && (ax > 0.0 || ay > 0.0) {
        // Tiny: scale up before squaring to avoid spurious underflow.
        ax = ldexp(ax, CM_SCALE_UP);
        let ay_s = ldexp(ay, CM_SCALE_UP);
        s = ldexp((ax + ax.hypot(ay_s)).sqrt(), CM_SCALE_DOWN);
    } else {
        ax /= 8.0;
        s = 2.0 * (ax + ax.hypot(ay / 8.0)).sqrt();
    }
    let d = ay / (2.0 * s);

    if x >= 0.0 {
        (s, copysign(d, y))
    } else {
        (d, copysign(s, y))
    }
}

// ── log helper (natural log core) ─────────────────────────────────────────────
fn clog(z: (f64, f64), errno: &mut Errno) -> (f64, f64) {
    let (x, y) = z;
    if !x.is_finite() || !y.is_finite() {
        const LOG_SPECIAL: [[(f64, f64); 7]; 7] = [
            [
                (INF, -P34),
                (INF, -P),
                (INF, -P),
                (INF, P),
                (INF, P),
                (INF, P34),
                (INF, N),
            ],
            [
                (INF, -P12),
                (U, U),
                (U, U),
                (U, U),
                (U, U),
                (INF, P12),
                (N, N),
            ],
            [
                (INF, -P12),
                (U, U),
                (-INF, -P),
                (-INF, P),
                (U, U),
                (INF, P12),
                (N, N),
            ],
            [
                (INF, -P12),
                (U, U),
                (-INF, -0.0),
                (-INF, 0.0),
                (U, U),
                (INF, P12),
                (N, N),
            ],
            [
                (INF, -P12),
                (U, U),
                (U, U),
                (U, U),
                (U, U),
                (INF, P12),
                (N, N),
            ],
            [
                (INF, -P14),
                (INF, -0.0),
                (INF, -0.0),
                (INF, 0.0),
                (INF, 0.0),
                (INF, P14),
                (INF, N),
            ],
            [(INF, N), (N, N), (N, N), (N, N), (N, N), (INF, N), (N, N)],
        ];
        return special_value(&LOG_SPECIAL, x, y);
    }
    let ax = x.abs();
    let ay = y.abs();

    let real;
    if ax > CM_LARGE_DOUBLE || ay > CM_LARGE_DOUBLE {
        real = (ax.hypot(ay) / 2.0).ln() + M_LN2;
    } else if ax < f64::MIN_POSITIVE && ay < f64::MIN_POSITIVE {
        if ax > 0.0 || ay > 0.0 {
            // catastrophic underflow region: scale up by 2^DBL_MANT_DIG.
            real = (ldexp(ax, 53).hypot(ldexp(ay, 53))).ln() - 53.0 * M_LN2;
        } else {
            // log(+/-0 +/-0)
            errno.edom = true;
            return (-INF, y.atan2(x));
        }
    } else {
        let h = ax * ax + ay * ay;
        if 0.71 <= h && h <= 1.73 {
            let am = ax.max(ay);
            let an = ax.min(ay);
            real = log1p((am - 1.0) * (am + 1.0) + an * an) / 2.0;
        } else {
            real = ax.hypot(ay).ln();
        }
    }
    (real, y.atan2(x))
}

// ── exp ──────────────────────────────────────────────────────────────────────
fn cexp(z: (f64, f64), errno: &mut Errno) -> (f64, f64) {
    let (x, y) = z;
    if !x.is_finite() || !y.is_finite() {
        // Imaginary part infinite/nan with finite real → result is nan+nanj,
        // raise EDOM (handled by the table giving nan and the check below).
        // Special case: exp(+-inf + yj) with finite nonzero y.
        if x.is_infinite() && y.is_finite() && y != 0.0 {
            let r = if x > 0.0 {
                (copysign(INF, y.cos()), copysign(INF, y.sin()))
            } else {
                (copysign(0.0, y.cos()), copysign(0.0, y.sin()))
            };
            return r;
        }
        const EXP_SPECIAL: [[(f64, f64); 7]; 7] = [
            [
                (0.0, 0.0),
                (U, U),
                (0.0, -0.0),
                (0.0, 0.0),
                (U, U),
                (0.0, 0.0),
                (0.0, 0.0),
            ],
            [(N, N), (U, U), (U, U), (U, U), (U, U), (N, N), (N, N)],
            [
                (N, N),
                (U, U),
                (1.0, -0.0),
                (1.0, 0.0),
                (U, U),
                (N, N),
                (N, N),
            ],
            [
                (N, N),
                (U, U),
                (1.0, -0.0),
                (1.0, 0.0),
                (U, U),
                (N, N),
                (N, N),
            ],
            [(N, N), (U, U), (U, U), (U, U), (U, U), (N, N), (N, N)],
            [
                (INF, N),
                (U, U),
                (INF, -0.0),
                (INF, 0.0),
                (U, U),
                (INF, N),
                (INF, N),
            ],
            [(N, N), (N, N), (N, -0.0), (N, 0.0), (N, N), (N, N), (N, N)],
        ];
        let r = special_value(&EXP_SPECIAL, x, y);
        // EDOM when imaginary part is +-inf and real part is finite or -inf.
        if y.is_infinite() && (x.is_finite() || (x.is_infinite() && x < 0.0)) {
            errno.edom = true;
        }
        return r;
    }

    let (rx, ry);
    if x > CM_LARGE_DOUBLE {
        // Scale to avoid overflow of exp(x): exp(x) = exp(x - LN2) * 2.
        let l = (x - M_LN2).exp();
        rx = l * y.cos() * 2.0;
        ry = l * y.sin() * 2.0;
    } else {
        let l = x.exp();
        rx = l * y.cos();
        ry = l * y.sin();
    }
    if rx.is_infinite() || ry.is_infinite() {
        errno.erange = true;
    }
    (rx, ry)
}

// ── log10 ────────────────────────────────────────────────────────────────────
fn clog10(z: (f64, f64), errno: &mut Errno) -> (f64, f64) {
    let (real, imag) = clog(z, errno);
    (real / M_LN10, imag / M_LN10)
}

// ── cosh ─────────────────────────────────────────────────────────────────────
fn ccosh(z: (f64, f64), errno: &mut Errno) -> (f64, f64) {
    let (x, y) = z;
    if !x.is_finite() || !y.is_finite() {
        const COSH_SPECIAL: [[(f64, f64); 7]; 7] = [
            [
                (INF, N),
                (U, U),
                (INF, 0.0),
                (INF, -0.0),
                (U, U),
                (INF, N),
                (INF, N),
            ],
            [(N, N), (U, U), (U, U), (U, U), (U, U), (N, N), (N, N)],
            [
                (N, 0.0),
                (U, U),
                (1.0, 0.0),
                (1.0, -0.0),
                (U, U),
                (N, 0.0),
                (N, 0.0),
            ],
            [
                (N, 0.0),
                (U, U),
                (1.0, -0.0),
                (1.0, 0.0),
                (U, U),
                (N, 0.0),
                (N, 0.0),
            ],
            [(N, N), (U, U), (U, U), (U, U), (U, U), (N, N), (N, N)],
            [
                (INF, N),
                (U, U),
                (INF, -0.0),
                (INF, 0.0),
                (U, U),
                (INF, N),
                (INF, N),
            ],
            [(N, N), (N, N), (N, 0.0), (N, 0.0), (N, N), (N, N), (N, N)],
        ];
        let r = special_value(&COSH_SPECIAL, x, y);
        // EDOM when y is +-inf and x is not nan; ERANGE when x is +-inf and y finite.
        if x.is_infinite() && y.is_finite() && y != 0.0 {
            errno.erange = true;
        } else if y.is_infinite() && !x.is_nan() {
            errno.edom = true;
        }
        return r;
    }
    if x.abs() > CM_LOG_LARGE_DOUBLE {
        // Avoid overflow: cosh(x) ~ exp(|x|)/2 for large |x|.
        let m = (x.abs() - M_LN2).exp();
        let rx = y.cos() * m;
        let ry = y.sin() * m * copysign(1.0, x);
        if rx.is_infinite() || ry.is_infinite() {
            errno.erange = true;
        }
        return (rx, ry);
    }
    let rx = y.cos() * x.cosh();
    let ry = y.sin() * x.sinh();
    if rx.is_infinite() || ry.is_infinite() {
        errno.erange = true;
    }
    (rx, ry)
}

// ── sinh ─────────────────────────────────────────────────────────────────────
fn csinh(z: (f64, f64), errno: &mut Errno) -> (f64, f64) {
    let (x, y) = z;
    if !x.is_finite() || !y.is_finite() {
        const SINH_SPECIAL: [[(f64, f64); 7]; 7] = [
            [
                (INF, N),
                (U, U),
                (-INF, -0.0),
                (-INF, 0.0),
                (U, U),
                (INF, N),
                (INF, N),
            ],
            [(N, N), (U, U), (U, U), (U, U), (U, U), (N, N), (N, N)],
            [
                (0.0, N),
                (U, U),
                (-0.0, -0.0),
                (-0.0, 0.0),
                (U, U),
                (0.0, N),
                (0.0, N),
            ],
            [
                (0.0, N),
                (U, U),
                (0.0, -0.0),
                (0.0, 0.0),
                (U, U),
                (0.0, N),
                (0.0, N),
            ],
            [(N, N), (U, U), (U, U), (U, U), (U, U), (N, N), (N, N)],
            [
                (INF, N),
                (U, U),
                (INF, -0.0),
                (INF, 0.0),
                (U, U),
                (INF, N),
                (INF, N),
            ],
            [(N, N), (N, N), (N, -0.0), (N, 0.0), (N, N), (N, N), (N, N)],
        ];
        let r = special_value(&SINH_SPECIAL, x, y);
        if x.is_infinite() && y.is_finite() && y != 0.0 {
            errno.erange = true;
        } else if y.is_infinite() && !x.is_nan() {
            errno.edom = true;
        }
        return r;
    }
    if x.abs() > CM_LOG_LARGE_DOUBLE {
        let m = (x.abs() - M_LN2).exp();
        let rx = y.cos() * m * copysign(1.0, x);
        let ry = y.sin() * m;
        if rx.is_infinite() || ry.is_infinite() {
            errno.erange = true;
        }
        return (rx, ry);
    }
    let rx = y.cos() * x.sinh();
    let ry = y.sin() * x.cosh();
    if rx.is_infinite() || ry.is_infinite() {
        errno.erange = true;
    }
    (rx, ry)
}

// ── tanh ─────────────────────────────────────────────────────────────────────
fn ctanh(z: (f64, f64), errno: &mut Errno) -> (f64, f64) {
    let (x, y) = z;
    if !x.is_finite() || !y.is_finite() {
        const TANH_SPECIAL: [[(f64, f64); 7]; 7] = [
            [
                (-1.0, 0.0),
                (U, U),
                (-1.0, -0.0),
                (-1.0, 0.0),
                (U, U),
                (-1.0, 0.0),
                (-1.0, 0.0),
            ],
            [(N, N), (U, U), (U, U), (U, U), (U, U), (N, N), (N, N)],
            [
                (N, N),
                (U, U),
                (-0.0, -0.0),
                (-0.0, 0.0),
                (U, U),
                (N, N),
                (N, N),
            ],
            [
                (N, N),
                (U, U),
                (0.0, -0.0),
                (0.0, 0.0),
                (U, U),
                (N, N),
                (N, N),
            ],
            [(N, N), (U, U), (U, U), (U, U), (U, U), (N, N), (N, N)],
            [
                (1.0, 0.0),
                (U, U),
                (1.0, -0.0),
                (1.0, 0.0),
                (U, U),
                (1.0, 0.0),
                (1.0, 0.0),
            ],
            [(N, N), (N, N), (N, -0.0), (N, 0.0), (N, N), (N, N), (N, N)],
        ];
        let r = special_value(&TANH_SPECIAL, x, y);
        if y.is_infinite() && x.is_finite() {
            errno.edom = true;
        }
        return r;
    }
    // Large |x|: tanh ~ +-1.
    if x.abs() > CM_LARGE_DOUBLE {
        let rx = copysign(1.0, x);
        let ry = 4.0 * y.sin() * y.cos() * (-2.0 * x.abs()).exp();
        return (rx, ry);
    }
    let tx = x.tanh();
    let ty = y.tan();
    let cx = 1.0 / x.cosh();
    let txty = tx * ty;
    let denom = 1.0 + txty * txty;
    let rx = tx * (1.0 + ty * ty) / denom;
    let ry = ((ty / denom) * cx) * cx;
    (rx, ry)
}

// Thresholds from CPython cmathmodule.c (computed at module init there).
const CM_SQRT_LARGE_DOUBLE: f64 = 9.480751908109176e+153; // sqrt(CM_LARGE_DOUBLE)
const CM_SQRT_DBL_MIN: f64 = 1.4916681462400413e-154; // sqrt(DBL_MIN)
const CM_LOG_LARGE_DOUBLE: f64 = 709.0895657128241; // log(CM_LARGE_DOUBLE)

// ── asinh ────────────────────────────────────────────────────────────────────
fn casinh(z: (f64, f64), errno: &mut Errno) -> (f64, f64) {
    let (x, y) = z;
    if !x.is_finite() || !y.is_finite() {
        const ASINH_SPECIAL: [[(f64, f64); 7]; 7] = [
            [
                (-INF, -P14),
                (-INF, -0.0),
                (-INF, -0.0),
                (-INF, 0.0),
                (-INF, 0.0),
                (-INF, P14),
                (-INF, N),
            ],
            [
                (-INF, -P12),
                (U, U),
                (U, U),
                (U, U),
                (U, U),
                (-INF, P12),
                (N, N),
            ],
            [
                (-INF, -P12),
                (U, U),
                (-0.0, -0.0),
                (-0.0, 0.0),
                (U, U),
                (-INF, P12),
                (N, N),
            ],
            [
                (INF, -P12),
                (U, U),
                (0.0, -0.0),
                (0.0, 0.0),
                (U, U),
                (INF, P12),
                (N, N),
            ],
            [
                (INF, -P12),
                (U, U),
                (U, U),
                (U, U),
                (U, U),
                (INF, P12),
                (N, N),
            ],
            [
                (INF, -P14),
                (INF, -0.0),
                (INF, -0.0),
                (INF, 0.0),
                (INF, 0.0),
                (INF, P14),
                (INF, N),
            ],
            [
                (INF, N),
                (N, N),
                (N, -0.0),
                (N, 0.0),
                (N, N),
                (INF, N),
                (N, N),
            ],
        ];
        let _ = errno;
        return special_value(&ASINH_SPECIAL, x, y);
    }
    if x.abs() > CM_SQRT_LARGE_DOUBLE || y.abs() > CM_SQRT_LARGE_DOUBLE {
        // real = copysign(log(hypot(x/2., y/2.))+M_LN2*2., z.real)
        let real = copysign((x / 2.0).hypot(y / 2.0).ln() + M_LN2 * 2.0, x);
        // imag = atan2(z.imag, fabs(z.real))
        let imag = y.atan2(x.abs());
        (real, imag)
    } else if x.abs() < CM_SQRT_DBL_MIN && y.abs() < CM_SQRT_DBL_MIN {
        // Small: asinh(z) ~ z; preserves underflow flag in CPython.
        (x, y)
    } else {
        // General case (CPython cmathmodule.c c_asinh):
        //   s1 = sqrt(1 + i*z),  s2 = sqrt(1 - i*z)
        //   real = asinh(s1.real*s2.imag - s2.real*s1.imag)
        //   imag = atan2(z.imag, s1.real*s2.real - s1.imag*s2.imag)
        let mut e = Errno::new();
        let s1 = csqrt((1.0 + y, -x), &mut e);
        let s2 = csqrt((1.0 - y, x), &mut e);
        let real = m_asinh(s1.0 * s2.1 - s2.0 * s1.1);
        let imag = y.atan2(s1.0 * s2.0 - s1.1 * s2.1);
        (real, imag)
    }
}

// ── asin ─────────────────────────────────────────────────────────────────────
fn casin(z: (f64, f64), errno: &mut Errno) -> (f64, f64) {
    // asin(z) = -i asinh(iz); CPython: s = asinh(-z.imag, z.real); (s.imag,-s.real)
    let s = casinh((-z.1, z.0), errno);
    (s.1, -s.0)
}

// ── acos ─────────────────────────────────────────────────────────────────────
fn cacos(z: (f64, f64), errno: &mut Errno) -> (f64, f64) {
    let (x, y) = z;
    if !x.is_finite() || !y.is_finite() {
        const ACOS_SPECIAL: [[(f64, f64); 7]; 7] = [
            [
                (P34, INF),
                (P, INF),
                (P, INF),
                (P, -INF),
                (P, -INF),
                (P34, -INF),
                (N, INF),
            ],
            [
                (P12, INF),
                (U, U),
                (U, U),
                (U, U),
                (U, U),
                (P12, -INF),
                (N, N),
            ],
            [
                (P12, INF),
                (U, U),
                (P12, 0.0),
                (P12, -0.0),
                (U, U),
                (P12, -INF),
                (P12, N),
            ],
            [
                (P12, INF),
                (U, U),
                (P12, 0.0),
                (P12, -0.0),
                (U, U),
                (P12, -INF),
                (P12, N),
            ],
            [
                (P12, INF),
                (U, U),
                (U, U),
                (U, U),
                (U, U),
                (P12, -INF),
                (N, N),
            ],
            [
                (P14, INF),
                (0.0, INF),
                (0.0, INF),
                (0.0, -INF),
                (0.0, -INF),
                (P14, -INF),
                (N, INF),
            ],
            [(N, INF), (N, N), (N, N), (N, N), (N, N), (N, -INF), (N, N)],
        ];
        let _ = errno;
        return special_value(&ACOS_SPECIAL, x, y);
    }
    if x.abs() > CM_LARGE_DOUBLE || y.abs() > CM_LARGE_DOUBLE {
        // r.real = atan2(fabs(z.imag), z.real)
        let real = y.abs().atan2(x);
        // r.imag = copysign(log(hypot(z.real/2., z.imag/2.))+M_LN2*2., -z.imag)
        let imag = copysign((x / 2.0).hypot(y / 2.0).ln() + M_LN2 * 2.0, -y);
        return (real, imag);
    }
    let mut e = Errno::new();
    let s1 = csqrt((1.0 - x, -y), &mut e);
    let s2 = csqrt((1.0 + x, y), &mut e);
    let real = 2.0 * (s1.0).atan2(s2.0);
    let imag = m_asinh(s2.0 * s1.1 - s2.1 * s1.0);
    (real, imag)
}

// ── acosh ────────────────────────────────────────────────────────────────────
fn cacosh(z: (f64, f64), errno: &mut Errno) -> (f64, f64) {
    let (x, y) = z;
    if !x.is_finite() || !y.is_finite() {
        const ACOSH_SPECIAL: [[(f64, f64); 7]; 7] = [
            [
                (INF, -P34),
                (INF, -P),
                (INF, -P),
                (INF, P),
                (INF, P),
                (INF, P34),
                (INF, N),
            ],
            [
                (INF, -P12),
                (U, U),
                (U, U),
                (U, U),
                (U, U),
                (INF, P12),
                (N, N),
            ],
            [
                (INF, -P12),
                (U, U),
                (0.0, -P12),
                (0.0, P12),
                (U, U),
                (INF, P12),
                (N, N),
            ],
            [
                (INF, -P12),
                (U, U),
                (0.0, -P12),
                (0.0, P12),
                (U, U),
                (INF, P12),
                (N, N),
            ],
            [
                (INF, -P12),
                (U, U),
                (U, U),
                (U, U),
                (U, U),
                (INF, P12),
                (N, N),
            ],
            [
                (INF, -P14),
                (INF, -0.0),
                (INF, -0.0),
                (INF, 0.0),
                (INF, 0.0),
                (INF, P14),
                (INF, N),
            ],
            [(INF, N), (N, N), (N, N), (N, N), (N, N), (INF, N), (N, N)],
        ];
        let _ = errno;
        return special_value(&ACOSH_SPECIAL, x, y);
    }
    if x.abs() > CM_LARGE_DOUBLE || y.abs() > CM_LARGE_DOUBLE {
        // r.real = log(hypot(z.real/2., z.imag/2.))+M_LN2*2.
        let real = (x / 2.0).hypot(y / 2.0).ln() + M_LN2 * 2.0;
        // r.imag = atan2(z.imag, z.real)
        let imag = y.atan2(x);
        return (real, imag);
    }
    let mut e = Errno::new();
    let s1 = csqrt((x - 1.0, y), &mut e);
    let s2 = csqrt((x + 1.0, y), &mut e);
    let real = m_asinh(s1.0 * s2.0 + s1.1 * s2.1);
    let imag = 2.0 * (s1.1).atan2(s2.0);
    (real, imag)
}

// ── atanh ────────────────────────────────────────────────────────────────────
fn catanh(z: (f64, f64), errno: &mut Errno) -> (f64, f64) {
    let (x, y) = z;
    // Reduce to the upper-right quadrant: atanh(-z) = -atanh(z),
    // atanh(conj z) = conj atanh(z).
    if x < 0.0 {
        let r = catanh((-x, -y), errno);
        return (-r.0, -r.1);
    }
    if !x.is_finite() || !y.is_finite() {
        const ATANH_SPECIAL: [[(f64, f64); 7]; 7] = [
            [
                (-0.0, -P12),
                (-0.0, -P12),
                (-0.0, -P12),
                (-0.0, P12),
                (-0.0, P12),
                (-0.0, P12),
                (-0.0, N),
            ],
            [
                (-0.0, -P12),
                (U, U),
                (U, U),
                (U, U),
                (U, U),
                (-0.0, P12),
                (N, N),
            ],
            [
                (-0.0, -P12),
                (U, U),
                (-0.0, -0.0),
                (-0.0, 0.0),
                (U, U),
                (-0.0, P12),
                (-0.0, N),
            ],
            [
                (0.0, -P12),
                (U, U),
                (0.0, -0.0),
                (0.0, 0.0),
                (U, U),
                (0.0, P12),
                (0.0, N),
            ],
            [
                (0.0, -P12),
                (U, U),
                (U, U),
                (U, U),
                (U, U),
                (0.0, P12),
                (N, N),
            ],
            [
                (0.0, -P12),
                (0.0, -P12),
                (0.0, -P12),
                (0.0, P12),
                (0.0, P12),
                (0.0, P12),
                (0.0, N),
            ],
            [
                (0.0, -P12),
                (N, N),
                (N, N),
                (N, N),
                (N, N),
                (0.0, P12),
                (N, N),
            ],
        ];
        let _ = errno;
        return special_value(&ATANH_SPECIAL, x, y);
    }
    // x >= 0 from here.
    let ay = y.abs();
    if x > CM_SQRT_LARGE_DOUBLE || ay > CM_SQRT_LARGE_DOUBLE {
        // For large |z|: h = hypot(ax/2., ay/2.); real = ax/4./h/h.
        let h = (x / 2.0).hypot(ay / 2.0);
        let real = x / 4.0 / h / h;
        let imag = -copysign(P12, -y);
        (real, imag)
    } else if x == 1.0 && ay < CM_SQRT_DBL_MIN {
        // atanh(1 + tiny*i): special branch (CPython cmathmodule.c).
        if ay == 0.0 {
            errno.edom = true;
            (INF, y)
        } else {
            // real = -log(sqrt(ay)/sqrt(hypot(ay, 2.)))
            let real = -(ay.sqrt() / ay.hypot(2.0).sqrt()).ln();
            // imag = copysign(atan2(2., -ay)/2., z.imag)
            let imag = copysign(2.0_f64.atan2(-ay) / 2.0, y);
            (real, imag)
        }
    } else {
        let real = log1p(4.0 * x / ((1.0 - x) * (1.0 - x) + ay * ay)) / 4.0;
        let imag = -(-2.0 * y).atan2((1.0 - x) * (1.0 + x) - ay * ay) / 2.0;
        (real, imag)
    }
}

// ── atan ─────────────────────────────────────────────────────────────────────
fn catan(z: (f64, f64), errno: &mut Errno) -> (f64, f64) {
    // atan(z) = -i atanh(iz); CPython: s = atanh(-z.imag, z.real); (s.imag,-s.real)
    let s = catanh((-z.1, z.0), errno);
    (s.1, -s.0)
}

// ── sin / cos / tan via hyperbolic ───────────────────────────────────────────
fn csin(z: (f64, f64), errno: &mut Errno) -> (f64, f64) {
    // sin(z) = -i sinh(iz)
    let s = csinh((-z.1, z.0), errno);
    (s.1, -s.0)
}
fn ccos(z: (f64, f64), errno: &mut Errno) -> (f64, f64) {
    // cos(z) = cosh(iz)
    ccosh((-z.1, z.0), errno)
}
fn ctan(z: (f64, f64), errno: &mut Errno) -> (f64, f64) {
    // tan(z) = -i tanh(iz)
    let s = ctanh((-z.1, z.0), errno);
    (s.1, -s.0)
}

// Raise the libm errno into a Python exception, or build the complex result.
fn finish(r: (f64, f64), errno: &Errno) -> MbValue {
    if errno.edom {
        return raise_value_error("math domain error");
    }
    if errno.erange {
        return raise_overflow_error("math range error");
    }
    make_complex(r.0, r.1)
}

fn raise_value_error(msg: &str) -> MbValue {
    super::super::exception::mb_raise(
        MbValue::from_ptr(MbObject::new_str("ValueError".to_string())),
        MbValue::from_ptr(MbObject::new_str(msg.to_string())),
    );
    MbValue::none()
}

fn raise_overflow_error(msg: &str) -> MbValue {
    super::super::exception::mb_raise(
        MbValue::from_ptr(MbObject::new_str("OverflowError".to_string())),
        MbValue::from_ptr(MbObject::new_str(msg.to_string())),
    );
    MbValue::none()
}

// ── Public surface — 24 cmath functions ──

// Trigonometric forward
pub fn mb_cmath_sin(z: MbValue) -> MbValue {
    let mut e = Errno::new();
    let r = csin(extract_complex(z), &mut e);
    finish(r, &e)
}

pub fn mb_cmath_cos(z: MbValue) -> MbValue {
    let mut e = Errno::new();
    let r = ccos(extract_complex(z), &mut e);
    finish(r, &e)
}

pub fn mb_cmath_tan(z: MbValue) -> MbValue {
    let mut e = Errno::new();
    let r = ctan(extract_complex(z), &mut e);
    finish(r, &e)
}

// Trigonometric inverse
pub fn mb_cmath_asin(z: MbValue) -> MbValue {
    let mut e = Errno::new();
    let r = casin(extract_complex(z), &mut e);
    finish(r, &e)
}

pub fn mb_cmath_acos(z: MbValue) -> MbValue {
    let mut e = Errno::new();
    let r = cacos(extract_complex(z), &mut e);
    finish(r, &e)
}

pub fn mb_cmath_atan(z: MbValue) -> MbValue {
    let mut e = Errno::new();
    let r = catan(extract_complex(z), &mut e);
    finish(r, &e)
}

// Hyperbolic forward
pub fn mb_cmath_sinh(z: MbValue) -> MbValue {
    let mut e = Errno::new();
    let r = csinh(extract_complex(z), &mut e);
    finish(r, &e)
}

pub fn mb_cmath_cosh(z: MbValue) -> MbValue {
    let mut e = Errno::new();
    let r = ccosh(extract_complex(z), &mut e);
    finish(r, &e)
}

pub fn mb_cmath_tanh(z: MbValue) -> MbValue {
    let mut e = Errno::new();
    let r = ctanh(extract_complex(z), &mut e);
    finish(r, &e)
}

// Hyperbolic inverse
pub fn mb_cmath_asinh(z: MbValue) -> MbValue {
    let mut e = Errno::new();
    let r = casinh(extract_complex(z), &mut e);
    finish(r, &e)
}

pub fn mb_cmath_acosh(z: MbValue) -> MbValue {
    let mut e = Errno::new();
    let r = cacosh(extract_complex(z), &mut e);
    finish(r, &e)
}

pub fn mb_cmath_atanh(z: MbValue) -> MbValue {
    let mut e = Errno::new();
    let r = catanh(extract_complex(z), &mut e);
    finish(r, &e)
}

// Exp / log
pub fn mb_cmath_exp(z: MbValue) -> MbValue {
    let mut e = Errno::new();
    let r = cexp(extract_complex(z), &mut e);
    finish(r, &e)
}

/// log(z, base=e). With base: cmath.log(x, base) = log(x) / log(base).
pub fn mb_cmath_log(z: MbValue, base: MbValue) -> MbValue {
    let mut e = Errno::new();
    let zl = clog(extract_complex(z), &mut e);
    if e.edom {
        return raise_value_error("math domain error");
    }
    if e.erange {
        return raise_overflow_error("math range error");
    }
    if base.is_none() {
        return make_complex(zl.0, zl.1);
    }
    let mut eb = Errno::new();
    let bl = clog(extract_complex(base), &mut eb);
    if eb.edom {
        return raise_value_error("math domain error");
    }
    if eb.erange {
        return raise_overflow_error("math range error");
    }
    let result = cdiv(zl, bl);
    make_complex(result.0, result.1)
}

pub fn mb_cmath_log10(z: MbValue) -> MbValue {
    let mut e = Errno::new();
    let r = clog10(extract_complex(z), &mut e);
    finish(r, &e)
}

pub fn mb_cmath_sqrt(z: MbValue) -> MbValue {
    // Strings can never be complex: CPython raises before any math.
    let is_str = z
        .as_ptr()
        .is_some_and(|p| unsafe { matches!((*p).data, ObjData::Str(_)) });
    if is_str {
        super::super::exception::mb_raise(
            MbValue::from_ptr(MbObject::new_str("TypeError".to_string())),
            MbValue::from_ptr(MbObject::new_str("must be a number, not str".to_string())),
        );
        return MbValue::none();
    }
    let mut e = Errno::new();
    let r = csqrt(extract_complex(z), &mut e);
    finish(r, &e)
}

// Complex division used only for log(z, base). Kept here for locality.
fn cdiv(a: (f64, f64), b: (f64, f64)) -> (f64, f64) {
    let d = b.0 * b.0 + b.1 * b.1;
    ((a.0 * b.0 + a.1 * b.1) / d, (a.1 * b.0 - a.0 * b.1) / d)
}

// Predicates
pub fn mb_cmath_isnan(z: MbValue) -> MbValue {
    let (r, i) = extract_complex(z);
    MbValue::from_bool(r.is_nan() || i.is_nan())
}

pub fn mb_cmath_isinf(z: MbValue) -> MbValue {
    let (r, i) = extract_complex(z);
    MbValue::from_bool(r.is_infinite() || i.is_infinite())
}

pub fn mb_cmath_isfinite(z: MbValue) -> MbValue {
    let (r, i) = extract_complex(z);
    MbValue::from_bool(r.is_finite() && i.is_finite())
}

/// cmath.isclose(a, b, rel_tol=1e-09, abs_tol=0.0). Matches when
/// |a - b| <= max(rel_tol * max(|a|, |b|), abs_tol). NaN never matches.
pub fn mb_cmath_isclose(a: MbValue, b: MbValue, rel_tol: f64, abs_tol: f64) -> MbValue {
    let (ar, ai) = extract_complex(a);
    let (br, bi) = extract_complex(b);
    if ar.is_nan() || ai.is_nan() || br.is_nan() || bi.is_nan() {
        return MbValue::from_bool(false);
    }
    if ar == br && ai == bi {
        return MbValue::from_bool(true);
    }
    let dr = ar - br;
    let di = ai - bi;
    // Use hypot for the complex magnitudes so inf/nan components match
    // CPython's c_abs (e.g. an infinite component dominates a NaN one).
    let diff = dr.hypot(di);
    let abs_a = ar.hypot(ai);
    let abs_b = br.hypot(bi);
    let tol = (rel_tol * abs_a.max(abs_b)).max(abs_tol);
    MbValue::from_bool(diff <= tol)
}

// Coordinate conversion
pub fn mb_cmath_phase(z: MbValue) -> MbValue {
    let (r, i) = extract_complex(z);
    MbValue::from_float(i.atan2(r))
}

pub fn mb_cmath_polar(z: MbValue) -> MbValue {
    let zc = extract_complex(z);
    // modulus = abs(z) via CPython c_abs (hypot with inf/nan special-casing:
    // an infinite component dominates a NaN one).
    let modulus = cabs(zc);
    let angle = zc.1.atan2(zc.0);
    MbValue::from_ptr(MbObject::new_tuple(vec![
        MbValue::from_float(modulus),
        MbValue::from_float(angle),
    ]))
}

pub fn mb_cmath_rect(r: MbValue, phi: MbValue) -> MbValue {
    let rv = as_f64(r).unwrap_or(0.0);
    let pv = as_f64(phi).unwrap_or(0.0);
    make_complex(rv * pv.cos(), rv * pv.sin())
}

// ── Tests ──

#[cfg(test)]
mod tests {
    use super::*;

    fn approx(a: f64, b: f64) -> bool {
        if a.is_nan() && b.is_nan() {
            return true;
        }
        (a - b).abs() < 1e-9
    }

    fn cz(z: MbValue) -> (f64, f64) {
        extract_complex(z)
    }

    #[test]
    fn test_sqrt_real_positive() {
        let (re, im) = cz(mb_cmath_sqrt(MbValue::from_float(4.0)));
        assert!(approx(re, 2.0));
        assert!(approx(im, 0.0));
    }

    #[test]
    fn test_sqrt_negative_yields_imaginary() {
        let (re, im) = cz(mb_cmath_sqrt(MbValue::from_float(-1.0)));
        assert!(approx(re, 0.0));
        assert!(approx(im, 1.0));
    }

    #[test]
    fn test_exp_i_pi_is_minus_one() {
        let (re, im) = cz(mb_cmath_exp(make_complex(0.0, std::f64::consts::PI)));
        assert!(approx(re, -1.0));
        assert!(im.abs() < 1e-9);
    }

    #[test]
    fn test_log_e_is_one() {
        let (re, im) = cz(mb_cmath_log(
            MbValue::from_float(std::f64::consts::E),
            MbValue::none(),
        ));
        assert!(approx(re, 1.0));
        assert!(approx(im, 0.0));
    }

    #[test]
    fn test_log10_of_100() {
        let (re, im) = cz(mb_cmath_log10(MbValue::from_float(100.0)));
        assert!(approx(re, 2.0));
        assert!(approx(im, 0.0));
    }

    #[test]
    fn test_sin_squared_plus_cos_squared_is_one() {
        let z = make_complex(0.5, 0.3);
        let (sr, si) = cz(mb_cmath_sin(z));
        let (cr, ci) = cz(mb_cmath_cos(z));
        let sin2 = cmul((sr, si), (sr, si));
        let cos2 = cmul((cr, ci), (cr, ci));
        let total = cadd(sin2, cos2);
        assert!(approx(total.0, 1.0));
        assert!(approx(total.1, 0.0));
    }

    #[test]
    fn test_asin_inverse_of_sin() {
        let z = make_complex(0.4, 0.0);
        let (re, im) = cz(mb_cmath_asin(mb_cmath_sin(z)));
        assert!(approx(re, 0.4));
        assert!(approx(im, 0.0));
    }

    #[test]
    fn test_atan_of_one_is_pi_over_4() {
        let (re, im) = cz(mb_cmath_atan(MbValue::from_float(1.0)));
        assert!(approx(re, std::f64::consts::FRAC_PI_4));
        assert!(approx(im, 0.0));
    }

    #[test]
    fn test_sinh_cosh_identity() {
        let z = make_complex(0.5, 0.3);
        let (sr, si) = cz(mb_cmath_sinh(z));
        let (cr, ci) = cz(mb_cmath_cosh(z));
        let sh2 = cmul((sr, si), (sr, si));
        let ch2 = cmul((cr, ci), (cr, ci));
        let diff = csub(ch2, sh2);
        assert!(approx(diff.0, 1.0));
        assert!(approx(diff.1, 0.0));
    }

    #[test]
    fn test_asinh_inverse_of_sinh() {
        let z = make_complex(0.3, 0.1);
        let (re, im) = cz(mb_cmath_asinh(mb_cmath_sinh(z)));
        assert!(approx(re, 0.3));
        assert!(approx(im, 0.1));
    }

    #[test]
    fn test_phase_of_i_is_pi_over_2() {
        let r = mb_cmath_phase(make_complex(0.0, 1.0));
        assert!(approx(r.as_float().unwrap(), std::f64::consts::FRAC_PI_2));
    }

    #[test]
    fn test_polar_rect_roundtrip() {
        let z = make_complex(3.0, 4.0);
        let polar = mb_cmath_polar(z);
        unsafe {
            if let ObjData::Tuple(ref items) = (*polar.as_ptr().unwrap()).data {
                let (re, im) = cz(mb_cmath_rect(items[0], items[1]));
                assert!(approx(re, 3.0));
                assert!(approx(im, 4.0));
            } else {
                panic!("polar did not return tuple");
            }
        }
    }

    #[test]
    fn test_isnan_detects_nan_imag() {
        assert_eq!(
            mb_cmath_isnan(make_complex(0.0, f64::NAN))
                .as_bool()
                .unwrap(),
            true
        );
    }

    #[test]
    fn test_isinf_detects_inf() {
        assert_eq!(
            mb_cmath_isinf(make_complex(f64::INFINITY, 0.0))
                .as_bool()
                .unwrap(),
            true
        );
    }

    #[test]
    fn test_isclose_exact_equal() {
        assert_eq!(
            mb_cmath_isclose(make_complex(1.0, 2.0), make_complex(1.0, 2.0), 1e-9, 0.0)
                .as_bool()
                .unwrap(),
            true,
        );
    }

    #[test]
    fn test_isclose_outside_tol() {
        assert_eq!(
            mb_cmath_isclose(make_complex(1.0, 0.0), make_complex(2.0, 0.0), 1e-9, 0.0)
                .as_bool()
                .unwrap(),
            false,
        );
    }
}
