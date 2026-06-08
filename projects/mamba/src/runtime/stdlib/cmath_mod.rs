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
    let z1 = a.first().copied().unwrap_or_else(MbValue::none);
    let z2 = a.get(1).copied().unwrap_or_else(MbValue::none);
    let rel_tol = a.get(2).and_then(|v| as_f64(*v)).unwrap_or(1e-9);
    let abs_tol = a.get(3).and_then(|v| as_f64(*v)).unwrap_or(0.0);
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

// ── Internal: complex arithmetic on (f64, f64) tuples ──

fn cadd(a: (f64, f64), b: (f64, f64)) -> (f64, f64) {
    (a.0 + b.0, a.1 + b.1)
}
fn csub(a: (f64, f64), b: (f64, f64)) -> (f64, f64) {
    (a.0 - b.0, a.1 - b.1)
}
fn cmul(a: (f64, f64), b: (f64, f64)) -> (f64, f64) {
    (a.0 * b.0 - a.1 * b.1, a.0 * b.1 + a.1 * b.0)
}
fn cdiv(a: (f64, f64), b: (f64, f64)) -> (f64, f64) {
    let d = b.0 * b.0 + b.1 * b.1;
    ((a.0 * b.0 + a.1 * b.1) / d, (a.1 * b.0 - a.0 * b.1) / d)
}
fn csqrt(z: (f64, f64)) -> (f64, f64) {
    let (r, i) = z;
    if r == 0.0 && i == 0.0 {
        return (0.0, 0.0);
    }
    let modulus = (r * r + i * i).sqrt();
    let real = ((modulus + r) / 2.0).sqrt();
    let imag = if i >= 0.0 {
        ((modulus - r) / 2.0).sqrt()
    } else {
        -((modulus - r) / 2.0).sqrt()
    };
    (real, imag)
}
fn clog(z: (f64, f64)) -> (f64, f64) {
    let (r, i) = z;
    let modulus = (r * r + i * i).sqrt();
    (modulus.ln(), i.atan2(r))
}

// ── Public surface — 24 cmath functions ──

// Trigonometric forward
pub fn mb_cmath_sin(z: MbValue) -> MbValue {
    let (r, i) = extract_complex(z);
    make_complex(r.sin() * i.cosh(), r.cos() * i.sinh())
}

pub fn mb_cmath_cos(z: MbValue) -> MbValue {
    let (r, i) = extract_complex(z);
    make_complex(r.cos() * i.cosh(), -(r.sin() * i.sinh()))
}

pub fn mb_cmath_tan(z: MbValue) -> MbValue {
    let (r, i) = extract_complex(z);
    let denom = (2.0 * r).cos() + (2.0 * i).cosh();
    make_complex((2.0 * r).sin() / denom, (2.0 * i).sinh() / denom)
}

// Trigonometric inverse
pub fn mb_cmath_asin(z: MbValue) -> MbValue {
    // asin(z) = -i * log(iz + sqrt(1 - z^2))
    let (a, b) = extract_complex(z);
    let z2 = cmul((a, b), (a, b));
    let one_minus = csub((1.0, 0.0), z2);
    let sqrt_term = csqrt(one_minus);
    let iz = (-b, a);
    let inside = cadd(iz, sqrt_term);
    let l = clog(inside);
    make_complex(l.1, -l.0)
}

pub fn mb_cmath_acos(z: MbValue) -> MbValue {
    // acos(z) = pi/2 - asin(z)
    let asin_z = mb_cmath_asin(z);
    let (ar, ai) = extract_complex(asin_z);
    make_complex(std::f64::consts::FRAC_PI_2 - ar, -ai)
}

pub fn mb_cmath_atan(z: MbValue) -> MbValue {
    // atan(z) = (i/2) * log((i + z) / (i - z))
    let (a, b) = extract_complex(z);
    let num = (a, 1.0 + b);
    let den = (-a, 1.0 - b);
    let ratio = cdiv(num, den);
    let l = clog(ratio);
    make_complex(-l.1 / 2.0, l.0 / 2.0)
}

// Hyperbolic forward
pub fn mb_cmath_sinh(z: MbValue) -> MbValue {
    let (r, i) = extract_complex(z);
    make_complex(r.sinh() * i.cos(), r.cosh() * i.sin())
}

pub fn mb_cmath_cosh(z: MbValue) -> MbValue {
    let (r, i) = extract_complex(z);
    make_complex(r.cosh() * i.cos(), r.sinh() * i.sin())
}

pub fn mb_cmath_tanh(z: MbValue) -> MbValue {
    let (r, i) = extract_complex(z);
    let denom = (2.0 * r).cosh() + (2.0 * i).cos();
    make_complex((2.0 * r).sinh() / denom, (2.0 * i).sin() / denom)
}

// Hyperbolic inverse
pub fn mb_cmath_asinh(z: MbValue) -> MbValue {
    let (a, b) = extract_complex(z);
    let z2 = cmul((a, b), (a, b));
    let one_plus = cadd((1.0, 0.0), z2);
    let s = csqrt(one_plus);
    let inside = cadd((a, b), s);
    let l = clog(inside);
    make_complex(l.0, l.1)
}

pub fn mb_cmath_acosh(z: MbValue) -> MbValue {
    let (a, b) = extract_complex(z);
    let z2 = cmul((a, b), (a, b));
    let minus_one = csub(z2, (1.0, 0.0));
    let s = csqrt(minus_one);
    let inside = cadd((a, b), s);
    let l = clog(inside);
    make_complex(l.0, l.1)
}

pub fn mb_cmath_atanh(z: MbValue) -> MbValue {
    let (a, b) = extract_complex(z);
    let num = (1.0 + a, b);
    let den = (1.0 - a, -b);
    let ratio = cdiv(num, den);
    let l = clog(ratio);
    make_complex(l.0 / 2.0, l.1 / 2.0)
}

// Exp / log
pub fn mb_cmath_exp(z: MbValue) -> MbValue {
    let (r, i) = extract_complex(z);
    let er = r.exp();
    make_complex(er * i.cos(), er * i.sin())
}

/// log(z, base=e). With base, log_base(z) = log(z) / log(base).
pub fn mb_cmath_log(z: MbValue, base: MbValue) -> MbValue {
    let zc = extract_complex(z);
    let zl = clog(zc);
    if base.is_none() {
        return make_complex(zl.0, zl.1);
    }
    let bc = extract_complex(base);
    if bc == (0.0, 0.0) {
        return make_complex(zl.0, zl.1);
    }
    let bl = clog(bc);
    let result = cdiv(zl, bl);
    make_complex(result.0, result.1)
}

pub fn mb_cmath_log10(z: MbValue) -> MbValue {
    let (r, i) = extract_complex(z);
    let zl = clog((r, i));
    let ln10 = 10f64.ln();
    make_complex(zl.0 / ln10, zl.1 / ln10)
}

pub fn mb_cmath_sqrt(z: MbValue) -> MbValue {
    let zc = extract_complex(z);
    let (r, i) = csqrt(zc);
    make_complex(r, i)
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
    let diff = (dr * dr + di * di).sqrt();
    let abs_a = (ar * ar + ai * ai).sqrt();
    let abs_b = (br * br + bi * bi).sqrt();
    let tol = (rel_tol * abs_a.max(abs_b)).max(abs_tol);
    MbValue::from_bool(diff <= tol)
}

// Coordinate conversion
pub fn mb_cmath_phase(z: MbValue) -> MbValue {
    let (r, i) = extract_complex(z);
    MbValue::from_float(i.atan2(r))
}

pub fn mb_cmath_polar(z: MbValue) -> MbValue {
    let (r, i) = extract_complex(z);
    let modulus = (r * r + i * i).sqrt();
    let angle = i.atan2(r);
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
