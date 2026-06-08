use super::super::rc::{MbObject, ObjData};
use super::super::value::MbValue;
/// math module for Mamba (#310 R3).
///
/// Provides: math.pi, math.e, math.inf, math.nan, math.sqrt(), math.floor(),
///           math.ceil(), math.sin(), math.cos(), math.tan(), math.log(),
///           math.exp(), math.pow(), math.fabs(), math.gcd(), math.factorial(),
///           math.isnan(), math.isinf(), math.isfinite(), math.lcm(), math.perm(),
///           math.sinh(), math.cosh(), math.modf(), math.frexp()
use std::collections::HashMap;

// ── Dispatch wrappers: native ABI ──

// Macro for unary dispatch wrappers
macro_rules! dispatch_unary {
    ($name:ident, $fn:ident) => {
        unsafe extern "C" fn $name(args_ptr: *const MbValue, nargs: usize) -> MbValue {
            let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
            $fn(a.get(0).copied().unwrap_or_else(MbValue::none))
        }
    };
}

macro_rules! dispatch_unary_number {
    ($name:ident, $fn:ident, $py_name:literal) => {
        unsafe extern "C" fn $name(args_ptr: *const MbValue, nargs: usize) -> MbValue {
            let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
            let arg = a.get(0).copied().unwrap_or_else(MbValue::none);
            if as_f64(arg).is_none() {
                return raise_type_error(concat!($py_name, "() argument must be a real number"));
            }
            $fn(arg)
        }
    };
}

// Macro for binary dispatch wrappers
macro_rules! dispatch_binary {
    ($name:ident, $fn:ident) => {
        unsafe extern "C" fn $name(args_ptr: *const MbValue, nargs: usize) -> MbValue {
            let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
            $fn(
                a.get(0).copied().unwrap_or_else(MbValue::none),
                a.get(1).copied().unwrap_or_else(MbValue::none),
            )
        }
    };
}

dispatch_unary_number!(dispatch_sqrt, mb_math_sqrt, "sqrt");
dispatch_unary!(dispatch_floor, mb_math_floor);
dispatch_unary!(dispatch_ceil, mb_math_ceil);
dispatch_unary!(dispatch_trunc, mb_math_trunc);
dispatch_unary!(dispatch_fabs, mb_math_fabs);
dispatch_unary!(dispatch_sin, mb_math_sin);
dispatch_unary!(dispatch_cos, mb_math_cos);
dispatch_unary!(dispatch_tan, mb_math_tan);
dispatch_unary!(dispatch_asin, mb_math_asin);
dispatch_unary!(dispatch_acos, mb_math_acos);
dispatch_unary!(dispatch_atan, mb_math_atan);
dispatch_unary!(dispatch_exp, mb_math_exp);
dispatch_unary!(dispatch_log2, mb_math_log2);
dispatch_unary!(dispatch_log10, mb_math_log10);
dispatch_unary!(dispatch_degrees, mb_math_degrees);
dispatch_unary!(dispatch_radians, mb_math_radians);
dispatch_unary!(dispatch_factorial, mb_math_factorial);
dispatch_unary!(dispatch_isnan, mb_math_isnan);
dispatch_unary!(dispatch_isinf, mb_math_isinf);
dispatch_unary_number!(dispatch_isfinite, mb_math_isfinite, "isfinite");
dispatch_unary!(dispatch_sinh, mb_math_sinh);
dispatch_unary!(dispatch_cosh, mb_math_cosh);
dispatch_unary!(dispatch_modf, mb_math_modf);
dispatch_unary!(dispatch_frexp, mb_math_frexp);

dispatch_binary!(dispatch_pow, mb_math_pow);
dispatch_binary!(dispatch_atan2, mb_math_atan2);
dispatch_binary!(dispatch_fmod, mb_math_fmod);
dispatch_binary!(dispatch_copysign, mb_math_copysign);
dispatch_binary!(dispatch_comb, mb_math_comb);
dispatch_binary!(dispatch_perm, mb_math_perm);
dispatch_binary!(dispatch_isclose, mb_math_isclose);
dispatch_unary!(dispatch_isqrt, mb_math_isqrt);
dispatch_unary!(dispatch_expm1, mb_math_expm1);
dispatch_unary!(dispatch_log1p, mb_math_log1p);
dispatch_unary!(dispatch_cbrt, mb_math_cbrt);
dispatch_unary!(dispatch_exp2, mb_math_exp2);
dispatch_unary!(dispatch_fsum, mb_math_fsum);
dispatch_unary!(dispatch_tanh, mb_math_tanh);
dispatch_unary!(dispatch_asinh, mb_math_asinh);
dispatch_unary!(dispatch_acosh, mb_math_acosh);
dispatch_unary!(dispatch_atanh, mb_math_atanh);
dispatch_unary!(dispatch_ulp, mb_math_ulp);
dispatch_binary!(dispatch_nextafter, mb_math_nextafter);
dispatch_binary!(dispatch_ldexp, mb_math_ldexp);
dispatch_binary!(dispatch_remainder, mb_math_remainder);
dispatch_binary!(dispatch_dist, mb_math_dist);

/// math.hypot(*coordinates) — Py3.8+ accepts N-D Euclidean norm.
/// Bare 2-arg form stays exact (`f64::hypot`); N-arg uses
/// `sqrt(sum(c**2 for c in coords))` with no overflow protection.
unsafe extern "C" fn dispatch_hypot(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    if nargs == 0 {
        return MbValue::from_float(0.0);
    }
    if nargs == 2 {
        return mb_math_hypot(a[0], a[1]);
    }
    let mut sum = 0.0f64;
    for v in a {
        match as_f64(*v) {
            Some(f) => sum += f * f,
            None => return MbValue::none(),
        }
    }
    MbValue::from_float(sum.sqrt())
}

/// math.prod(iterable, *, start=1) — Py3.8+ multiplicative reduction.
/// Trailing kwargs dict (lowering convention) carries `start`.
unsafe extern "C" fn dispatch_prod(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    let iterable = a.get(0).copied().unwrap_or_else(MbValue::none);
    // Detect trailing kwargs Dict for start=.
    let start = if nargs >= 2 {
        let last = a[nargs - 1];
        if let Some(ptr) = last.as_ptr() {
            unsafe {
                if let ObjData::Dict(ref lock) = (*ptr).data {
                    let guard = lock.read().unwrap();
                    let key = super::super::dict_ops::DictKey::Str("start".to_string());
                    guard
                        .get(&key)
                        .copied()
                        .unwrap_or_else(|| MbValue::from_int(1))
                } else {
                    a[1]
                }
            }
        } else {
            a[1]
        }
    } else {
        MbValue::from_int(1)
    };
    mb_math_prod(iterable, start)
}

/// math.gcd(*integers) — Python 3.9+ accepts any number of int args.
/// gcd() == 0; gcd(x) == |x|; multi-arg reduces left-to-right.
unsafe extern "C" fn dispatch_gcd(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    if a.is_empty() {
        return MbValue::from_int(0);
    }
    let mut acc = mb_math_gcd(a[0], MbValue::from_int(0));
    for v in &a[1..] {
        acc = mb_math_gcd(acc, *v);
    }
    acc
}

/// math.lcm(*integers) — Python 3.9+ variadic. lcm() == 1; lcm(x) == |x|.
/// Any zero argument short-circuits to 0.
unsafe extern "C" fn dispatch_lcm(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    if a.is_empty() {
        return MbValue::from_int(1);
    }
    let mut acc = mb_math_lcm(
        a[0],
        MbValue::from_int(a[0].as_int().map(|i| i).unwrap_or(0)),
    );
    // lcm(x) = |x|; reproduce by abs().
    if let Some(x) = a[0].as_int() {
        acc = MbValue::from_int(x.abs());
    }
    for v in &a[1..] {
        acc = mb_math_lcm(acc, *v);
        if acc.as_int() == Some(0) {
            return acc;
        }
    }
    acc
}

/// math.log(x) or math.log(x, base) -- variable arity
unsafe extern "C" fn dispatch_log(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    let x = a.get(0).copied().unwrap_or_else(MbValue::none);
    if nargs >= 2 {
        let base = a.get(1).copied().unwrap_or_else(MbValue::none);
        mb_math_log_base(x, base)
    } else {
        mb_math_log(x)
    }
}

/// Register the math module.
pub fn register() {
    let mut attrs = HashMap::new();

    // Constants
    attrs.insert("pi".to_string(), MbValue::from_float(std::f64::consts::PI));
    attrs.insert("e".to_string(), MbValue::from_float(std::f64::consts::E));
    attrs.insert(
        "tau".to_string(),
        MbValue::from_float(std::f64::consts::TAU),
    );
    attrs.insert("inf".to_string(), MbValue::from_float(f64::INFINITY));
    attrs.insert("nan".to_string(), MbValue::from_float(f64::NAN));

    // Functions: use dispatchers array + register in NATIVE_FUNC_ADDRS
    let dispatchers: Vec<(&str, usize)> = vec![
        ("sqrt", dispatch_sqrt as *const () as usize),
        ("floor", dispatch_floor as *const () as usize),
        ("ceil", dispatch_ceil as *const () as usize),
        ("trunc", dispatch_trunc as *const () as usize),
        ("fabs", dispatch_fabs as *const () as usize),
        ("sin", dispatch_sin as *const () as usize),
        ("cos", dispatch_cos as *const () as usize),
        ("tan", dispatch_tan as *const () as usize),
        ("asin", dispatch_asin as *const () as usize),
        ("acos", dispatch_acos as *const () as usize),
        ("atan", dispatch_atan as *const () as usize),
        ("exp", dispatch_exp as *const () as usize),
        ("log", dispatch_log as *const () as usize),
        ("log2", dispatch_log2 as *const () as usize),
        ("log10", dispatch_log10 as *const () as usize),
        ("degrees", dispatch_degrees as *const () as usize),
        ("radians", dispatch_radians as *const () as usize),
        ("factorial", dispatch_factorial as *const () as usize),
        ("isnan", dispatch_isnan as *const () as usize),
        ("isinf", dispatch_isinf as *const () as usize),
        ("isfinite", dispatch_isfinite as *const () as usize),
        ("sinh", dispatch_sinh as *const () as usize),
        ("cosh", dispatch_cosh as *const () as usize),
        ("modf", dispatch_modf as *const () as usize),
        ("frexp", dispatch_frexp as *const () as usize),
        ("pow", dispatch_pow as *const () as usize),
        ("atan2", dispatch_atan2 as *const () as usize),
        ("fmod", dispatch_fmod as *const () as usize),
        ("copysign", dispatch_copysign as *const () as usize),
        ("hypot", dispatch_hypot as *const () as usize),
        ("gcd", dispatch_gcd as *const () as usize),
        ("lcm", dispatch_lcm as *const () as usize),
        ("comb", dispatch_comb as *const () as usize),
        ("perm", dispatch_perm as *const () as usize),
        ("isclose", dispatch_isclose as *const () as usize),
        ("isqrt", dispatch_isqrt as *const () as usize),
        ("expm1", dispatch_expm1 as *const () as usize),
        ("log1p", dispatch_log1p as *const () as usize),
        ("ldexp", dispatch_ldexp as *const () as usize),
        ("remainder", dispatch_remainder as *const () as usize),
        ("dist", dispatch_dist as *const () as usize),
        ("prod", dispatch_prod as *const () as usize),
        ("cbrt", dispatch_cbrt as *const () as usize),
        ("exp2", dispatch_exp2 as *const () as usize),
        ("fsum", dispatch_fsum as *const () as usize),
        ("tanh", dispatch_tanh as *const () as usize),
        ("asinh", dispatch_asinh as *const () as usize),
        ("acosh", dispatch_acosh as *const () as usize),
        ("atanh", dispatch_atanh as *const () as usize),
        ("nextafter", dispatch_nextafter as *const () as usize),
        ("ulp", dispatch_ulp as *const () as usize),
    ];
    for (name, addr) in dispatchers {
        attrs.insert(name.to_string(), MbValue::from_func(addr));
        super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
            s.borrow_mut().insert(addr as u64);
        });
    }

    super::register_module("math", attrs);
}

// ── Helper: extract numeric value as f64 ──

fn as_f64(val: MbValue) -> Option<f64> {
    if let Some(f) = val.as_float() {
        Some(f)
    } else if let Some(i) = val.as_int() {
        Some(i as f64)
    } else {
        None
    }
}

fn as_i64(val: MbValue) -> Option<i64> {
    if let Some(i) = val.as_int() {
        Some(i)
    } else if let Some(f) = val.as_float() {
        Some(f as i64)
    } else {
        None
    }
}

fn raise_type_error(msg: &str) -> MbValue {
    super::super::exception::mb_raise(
        MbValue::from_ptr(MbObject::new_str("TypeError".to_string())),
        MbValue::from_ptr(MbObject::new_str(msg.to_string())),
    );
    MbValue::none()
}

// ── Unary math functions ──

pub fn mb_math_sqrt(val: MbValue) -> MbValue {
    match as_f64(val) {
        Some(f) if f < 0.0 => {
            super::super::exception::mb_raise(
                MbValue::from_ptr(MbObject::new_str("ValueError".to_string())),
                MbValue::from_ptr(MbObject::new_str("math domain error".to_string())),
            );
            MbValue::none()
        }
        Some(f) => MbValue::from_float(f.sqrt()),
        None => MbValue::none(),
    }
}

pub fn mb_math_floor(val: MbValue) -> MbValue {
    as_f64(val)
        .map(|f| MbValue::from_int(f.floor() as i64))
        .unwrap_or(MbValue::none())
}

pub fn mb_math_ceil(val: MbValue) -> MbValue {
    as_f64(val)
        .map(|f| MbValue::from_int(f.ceil() as i64))
        .unwrap_or(MbValue::none())
}

pub fn mb_math_trunc(val: MbValue) -> MbValue {
    as_f64(val)
        .map(|f| MbValue::from_int(f.trunc() as i64))
        .unwrap_or(MbValue::none())
}

pub fn mb_math_fabs(val: MbValue) -> MbValue {
    as_f64(val)
        .map(|f| MbValue::from_float(f.abs()))
        .unwrap_or(MbValue::none())
}

pub fn mb_math_sin(val: MbValue) -> MbValue {
    as_f64(val)
        .map(|f| MbValue::from_float(f.sin()))
        .unwrap_or(MbValue::none())
}

pub fn mb_math_cos(val: MbValue) -> MbValue {
    as_f64(val)
        .map(|f| MbValue::from_float(f.cos()))
        .unwrap_or(MbValue::none())
}

pub fn mb_math_tan(val: MbValue) -> MbValue {
    as_f64(val)
        .map(|f| MbValue::from_float(f.tan()))
        .unwrap_or(MbValue::none())
}

pub fn mb_math_asin(val: MbValue) -> MbValue {
    as_f64(val)
        .map(|f| MbValue::from_float(f.asin()))
        .unwrap_or(MbValue::none())
}

pub fn mb_math_acos(val: MbValue) -> MbValue {
    as_f64(val)
        .map(|f| MbValue::from_float(f.acos()))
        .unwrap_or(MbValue::none())
}

pub fn mb_math_atan(val: MbValue) -> MbValue {
    as_f64(val)
        .map(|f| MbValue::from_float(f.atan()))
        .unwrap_or(MbValue::none())
}

pub fn mb_math_exp(val: MbValue) -> MbValue {
    as_f64(val)
        .map(|f| MbValue::from_float(f.exp()))
        .unwrap_or(MbValue::none())
}

pub fn mb_math_log(val: MbValue) -> MbValue {
    match as_f64(val) {
        Some(f) if f <= 0.0 => {
            super::super::exception::mb_raise(
                MbValue::from_ptr(MbObject::new_str("ValueError".to_string())),
                MbValue::from_ptr(MbObject::new_str("math domain error".to_string())),
            );
            MbValue::none()
        }
        Some(f) => MbValue::from_float(f.ln()),
        None => MbValue::none(),
    }
}

pub fn mb_math_log2(val: MbValue) -> MbValue {
    as_f64(val)
        .map(|f| MbValue::from_float(f.log2()))
        .unwrap_or(MbValue::none())
}

pub fn mb_math_log10(val: MbValue) -> MbValue {
    as_f64(val)
        .map(|f| MbValue::from_float(f.log10()))
        .unwrap_or(MbValue::none())
}

pub fn mb_math_degrees(val: MbValue) -> MbValue {
    as_f64(val)
        .map(|f| MbValue::from_float(f.to_degrees()))
        .unwrap_or(MbValue::none())
}

pub fn mb_math_radians(val: MbValue) -> MbValue {
    as_f64(val)
        .map(|f| MbValue::from_float(f.to_radians()))
        .unwrap_or(MbValue::none())
}

// ── Binary math functions ──

pub fn mb_math_pow(base: MbValue, exp: MbValue) -> MbValue {
    match (as_f64(base), as_f64(exp)) {
        (Some(b), Some(e)) => MbValue::from_float(b.powf(e)),
        _ => MbValue::none(),
    }
}

pub fn mb_math_atan2(y: MbValue, x: MbValue) -> MbValue {
    match (as_f64(y), as_f64(x)) {
        (Some(yv), Some(xv)) => MbValue::from_float(yv.atan2(xv)),
        _ => MbValue::none(),
    }
}

pub fn mb_math_fmod(x: MbValue, y: MbValue) -> MbValue {
    match (as_f64(x), as_f64(y)) {
        (Some(xv), Some(yv)) => MbValue::from_float(xv % yv),
        _ => MbValue::none(),
    }
}

pub fn mb_math_copysign(x: MbValue, y: MbValue) -> MbValue {
    match (as_f64(x), as_f64(y)) {
        (Some(xv), Some(yv)) => MbValue::from_float(xv.copysign(yv)),
        _ => MbValue::none(),
    }
}

pub fn mb_math_hypot(x: MbValue, y: MbValue) -> MbValue {
    match (as_f64(x), as_f64(y)) {
        (Some(xv), Some(yv)) => MbValue::from_float(xv.hypot(yv)),
        _ => MbValue::none(),
    }
}

// ── Integer math ──

pub fn mb_math_gcd(a: MbValue, b: MbValue) -> MbValue {
    match (as_i64(a), as_i64(b)) {
        (Some(mut x), Some(mut y)) => {
            x = x.abs();
            y = y.abs();
            while y != 0 {
                let t = y;
                y = x % y;
                x = t;
            }
            MbValue::from_int(x)
        }
        _ => MbValue::none(),
    }
}

/// math.isqrt(n) — integer square root, Python 3.8+. Returns ⌊√n⌋ for n ≥ 0;
/// raises ValueError for negative input. Uses Newton's method on i64.
pub fn mb_math_isqrt(val: MbValue) -> MbValue {
    let Some(n) = as_i64(val) else {
        return MbValue::none();
    };
    if n < 0 {
        super::super::exception::mb_raise(
            MbValue::from_ptr(MbObject::new_str("ValueError".to_string())),
            MbValue::from_ptr(MbObject::new_str(
                "isqrt() argument must be nonnegative".to_string(),
            )),
        );
        return MbValue::none();
    }
    if n < 2 {
        return MbValue::from_int(n);
    }
    // Newton's method seeded with the float sqrt; refine in i64 space.
    let mut x = (n as f64).sqrt() as i64;
    // Two correction iterations cover any rounding inaccuracy from f64.
    for _ in 0..2 {
        x = (x + n / x) / 2;
    }
    while x * x > n {
        x -= 1;
    }
    while (x + 1).checked_mul(x + 1).map_or(false, |sq| sq <= n) {
        x += 1;
    }
    MbValue::from_int(x)
}

pub fn mb_math_factorial(val: MbValue) -> MbValue {
    // CPython 3.12: factorial only accepts non-negative integers.
    // Float arguments raise TypeError, negative ints raise ValueError.
    if val.as_float().is_some() && val.as_int().is_none() {
        super::super::exception::mb_raise(
            MbValue::from_ptr(MbObject::new_str("TypeError".to_string())),
            MbValue::from_ptr(MbObject::new_str(
                "'float' object cannot be interpreted as an integer".to_string(),
            )),
        );
        return MbValue::none();
    }
    if let Some(n) = val.as_int() {
        if n < 0 {
            super::super::exception::mb_raise(
                MbValue::from_ptr(MbObject::new_str("ValueError".to_string())),
                MbValue::from_ptr(MbObject::new_str(
                    "factorial() not defined for negative values".to_string(),
                )),
            );
            return MbValue::none();
        }
        if n > 20 {
            return MbValue::none();
        } // Overflow for i64
        let result: i64 = (1..=n).product();
        MbValue::from_int(result)
    } else {
        MbValue::none()
    }
}

pub fn mb_math_comb(n: MbValue, k: MbValue) -> MbValue {
    match (as_i64(n), as_i64(k)) {
        (Some(n), Some(k)) if n >= 0 && k >= 0 && k <= n => {
            let k = k.min(n - k) as u64;
            let n = n as u64;
            let mut result: u64 = 1;
            for i in 0..k {
                result = result * (n - i) / (i + 1);
            }
            MbValue::from_int(result as i64)
        }
        _ => MbValue::from_int(0),
    }
}

// ── Additional functions ──

pub fn mb_math_lcm(a: MbValue, b: MbValue) -> MbValue {
    match (as_i64(a), as_i64(b)) {
        (Some(x), Some(y)) => {
            if x == 0 || y == 0 {
                MbValue::from_int(0)
            } else {
                let ax = x.abs();
                let ay = y.abs();
                // lcm(a,b) = |a*b| / gcd(a,b)
                let mut gx = ax;
                let mut gy = ay;
                while gy != 0 {
                    let t = gy;
                    gy = gx % gy;
                    gx = t;
                }
                MbValue::from_int(ax / gx * ay)
            }
        }
        _ => MbValue::none(),
    }
}

pub fn mb_math_perm(n: MbValue, k: MbValue) -> MbValue {
    match (as_i64(n), as_i64(k)) {
        (Some(n), Some(k)) if n >= 0 && k >= 0 && k <= n => {
            let mut result: i64 = 1;
            for i in 0..k {
                result *= n - i;
            }
            MbValue::from_int(result)
        }
        _ => MbValue::from_int(0),
    }
}

pub fn mb_math_sinh(val: MbValue) -> MbValue {
    as_f64(val)
        .map(|f| MbValue::from_float(f.sinh()))
        .unwrap_or(MbValue::none())
}

pub fn mb_math_cosh(val: MbValue) -> MbValue {
    as_f64(val)
        .map(|f| MbValue::from_float(f.cosh()))
        .unwrap_or(MbValue::none())
}

/// math.log(x, base) -> log_base(x)
pub fn mb_math_log_base(val: MbValue, base: MbValue) -> MbValue {
    match (as_f64(val), as_f64(base)) {
        (Some(x), Some(_)) if x <= 0.0 => {
            super::super::exception::mb_raise(
                MbValue::from_ptr(MbObject::new_str("ValueError".to_string())),
                MbValue::from_ptr(MbObject::new_str("math domain error".to_string())),
            );
            MbValue::none()
        }
        (Some(x), Some(b)) => MbValue::from_float(x.ln() / b.ln()),
        _ => MbValue::none(),
    }
}

/// math.modf(x) -> (fractional, integer) as tuple
pub fn mb_math_modf(val: MbValue) -> MbValue {
    match as_f64(val) {
        Some(f) => {
            let trunc = f.trunc();
            let frac = f - trunc;
            MbValue::from_ptr(MbObject::new_tuple(vec![
                MbValue::from_float(frac),
                MbValue::from_float(trunc),
            ]))
        }
        None => MbValue::none(),
    }
}

/// math.frexp(x) -> (mantissa, exponent) as tuple
pub fn mb_math_frexp(val: MbValue) -> MbValue {
    match as_f64(val) {
        Some(f) => {
            if f == 0.0 {
                return MbValue::from_ptr(MbObject::new_tuple(vec![
                    MbValue::from_float(0.0),
                    MbValue::from_int(0),
                ]));
            }
            // frexp: f = m * 2^e where 0.5 <= |m| < 1.0
            let bits = f.to_bits();
            let sign = if (bits >> 63) != 0 { -1.0f64 } else { 1.0f64 };
            let exponent = ((bits >> 52) & 0x7FF) as i64;
            let mantissa_bits = bits & 0x000FFFFFFFFFFFFF;

            if exponent == 0 {
                // Subnormal
                let normalized = f * (1u64 << 52) as f64;
                let nbits = normalized.to_bits();
                let nexp = ((nbits >> 52) & 0x7FF) as i64;
                let m_bits = nbits & 0x000FFFFFFFFFFFFF;
                let m = f64::from_bits(0x3FE0000000000000 | m_bits) * sign;
                let e = nexp - 1022 - 52;
                MbValue::from_ptr(MbObject::new_tuple(vec![
                    MbValue::from_float(m),
                    MbValue::from_int(e),
                ]))
            } else {
                let m = f64::from_bits(0x3FE0000000000000 | mantissa_bits) * sign;
                let e = exponent - 1022;
                MbValue::from_ptr(MbObject::new_tuple(vec![
                    MbValue::from_float(m),
                    MbValue::from_int(e),
                ]))
            }
        }
        None => MbValue::none(),
    }
}

// ── Predicates ──

pub fn mb_math_isnan(val: MbValue) -> MbValue {
    MbValue::from_bool(val.as_float().map(|f| f.is_nan()).unwrap_or(false))
}

pub fn mb_math_isinf(val: MbValue) -> MbValue {
    MbValue::from_bool(val.as_float().map(|f| f.is_infinite()).unwrap_or(false))
}

pub fn mb_math_isfinite(val: MbValue) -> MbValue {
    MbValue::from_bool(val.as_float().map(|f| f.is_finite()).unwrap_or(true))
}

pub fn mb_math_isclose(a: MbValue, b: MbValue) -> MbValue {
    match (as_f64(a), as_f64(b)) {
        (Some(x), Some(y)) => {
            let rel_tol: f64 = 1e-9;
            let abs_tol: f64 = 0.0;
            let diff = (x - y).abs();
            MbValue::from_bool(diff <= f64::max(abs_tol, rel_tol * f64::max(x.abs(), y.abs())))
        }
        _ => MbValue::from_bool(false),
    }
}

// ── Py3.8+ additions ──

pub fn mb_math_expm1(val: MbValue) -> MbValue {
    match as_f64(val) {
        Some(f) => MbValue::from_float(f.exp_m1()),
        None => MbValue::none(),
    }
}

pub fn mb_math_log1p(val: MbValue) -> MbValue {
    match as_f64(val) {
        Some(f) => MbValue::from_float(f.ln_1p()),
        None => MbValue::none(),
    }
}

pub fn mb_math_ldexp(x: MbValue, i: MbValue) -> MbValue {
    match (as_f64(x), as_i64(i)) {
        (Some(xv), Some(iv)) => MbValue::from_float(xv * 2.0f64.powi(iv as i32)),
        _ => MbValue::none(),
    }
}

/// IEEE 754 remainder: r = x - n*y with n = round-to-even of x/y.
/// Differs from `fmod` (truncated remainder) — required by CPython
/// `math.remainder`. Built on the bit-level rounding shape: shift x/y
/// to identify the nearest integer and break ties to even.
pub fn mb_math_remainder(x: MbValue, y: MbValue) -> MbValue {
    match (as_f64(x), as_f64(y)) {
        (Some(xv), Some(yv)) => {
            if yv == 0.0 || xv.is_infinite() {
                return MbValue::from_float(f64::NAN);
            }
            let q = xv / yv;
            let n_floor = q.floor();
            let frac = q - n_floor;
            let n = if frac < 0.5 {
                n_floor
            } else if frac > 0.5 {
                n_floor + 1.0
            } else {
                // tie → round-to-even
                let candidate_lo = n_floor;
                let candidate_hi = n_floor + 1.0;
                if (candidate_lo as i64) & 1 == 0 {
                    candidate_lo
                } else {
                    candidate_hi
                }
            };
            MbValue::from_float(xv - n * yv)
        }
        _ => MbValue::none(),
    }
}

/// math.dist(p, q) — Euclidean distance between two same-length point
/// sequences (lists or tuples). Mismatched lengths return None for now;
/// CPython raises ValueError, but the surrounding contract here is
/// best-effort until a math-level error path lands.
pub fn mb_math_dist(p: MbValue, q: MbValue) -> MbValue {
    let pv = extract_seq_floats(p);
    let qv = extract_seq_floats(q);
    let (pv, qv) = match (pv, qv) {
        (Some(a), Some(b)) if a.len() == b.len() => (a, b),
        _ => return MbValue::none(),
    };
    let mut sum = 0.0f64;
    for (a, b) in pv.iter().zip(qv.iter()) {
        let d = a - b;
        sum += d * d;
    }
    MbValue::from_float(sum.sqrt())
}

/// math.prod(iterable, start=1) — multiplicative reduction. Stays
/// integer-typed when `start` and every element are ints; promotes to
/// float as soon as a float appears. Mirrors CPython's
/// "preserve int when possible" rule.
pub fn mb_math_prod(iterable: MbValue, start: MbValue) -> MbValue {
    let items = match iterable.as_ptr() {
        Some(ptr) => unsafe {
            match &(*ptr).data {
                ObjData::List(ref lock) => lock.read().unwrap().to_vec(),
                ObjData::Tuple(items) => items.clone(),
                _ => return MbValue::none(),
            }
        },
        None => return MbValue::none(),
    };

    let mut promoted = start.as_float().is_some() && start.as_int().is_none();
    let mut acc_i: i64 = start.as_int().unwrap_or(1);
    let mut acc_f: f64 = start.as_float().unwrap_or(acc_i as f64);

    for v in items {
        if let Some(i) = v.as_int() {
            if promoted {
                acc_f *= i as f64;
            } else {
                acc_i = acc_i.saturating_mul(i);
            }
        } else if let Some(f) = v.as_float() {
            if !promoted {
                acc_f = acc_i as f64;
                promoted = true;
            }
            acc_f *= f;
        } else {
            return MbValue::none();
        }
    }
    if promoted {
        MbValue::from_float(acc_f)
    } else {
        MbValue::from_int(acc_i)
    }
}

// math.tanh / asinh / acosh / atanh — hyperbolic + inverse-hyperbolic.
pub fn mb_math_tanh(val: MbValue) -> MbValue {
    as_f64(val)
        .map(|f| MbValue::from_float(f.tanh()))
        .unwrap_or(MbValue::none())
}

pub fn mb_math_asinh(val: MbValue) -> MbValue {
    as_f64(val)
        .map(|f| MbValue::from_float(f.asinh()))
        .unwrap_or(MbValue::none())
}

pub fn mb_math_acosh(val: MbValue) -> MbValue {
    match as_f64(val) {
        Some(f) if f < 1.0 => {
            super::super::exception::mb_raise(
                MbValue::from_ptr(MbObject::new_str("ValueError".to_string())),
                MbValue::from_ptr(MbObject::new_str("math domain error".to_string())),
            );
            MbValue::none()
        }
        Some(f) => MbValue::from_float(f.acosh()),
        None => MbValue::none(),
    }
}

pub fn mb_math_atanh(val: MbValue) -> MbValue {
    match as_f64(val) {
        Some(f) if f.abs() >= 1.0 => {
            super::super::exception::mb_raise(
                MbValue::from_ptr(MbObject::new_str("ValueError".to_string())),
                MbValue::from_ptr(MbObject::new_str("math domain error".to_string())),
            );
            MbValue::none()
        }
        Some(f) => MbValue::from_float(f.atanh()),
        None => MbValue::none(),
    }
}

// math.nextafter(x, y) — Py3.9+. Next representable f64 from x toward y.
pub fn mb_math_nextafter(x: MbValue, y: MbValue) -> MbValue {
    let (x, y) = match (as_f64(x), as_f64(y)) {
        (Some(a), Some(b)) => (a, b),
        _ => return MbValue::none(),
    };
    if x.is_nan() || y.is_nan() {
        return MbValue::from_float(f64::NAN);
    }
    let r = if x < y {
        x.next_up()
    } else if x > y {
        x.next_down()
    } else {
        // x == y: CPython returns y (preserves sign of zero).
        y
    };
    MbValue::from_float(r)
}

// math.ulp(x) — Py3.9+. Value of the least significant bit of x.
//   ulp(NaN)   = NaN
//   ulp(±inf)  = +inf
//   ulp(0)     = smallest positive subnormal
//   ulp(finite, |x| at boundary): take the larger neighbor distance
pub fn mb_math_ulp(val: MbValue) -> MbValue {
    let x = match as_f64(val) {
        Some(f) => f,
        None => return MbValue::none(),
    };
    let r = if x.is_nan() {
        f64::NAN
    } else if x.is_infinite() {
        f64::INFINITY
    } else {
        let ax = x.abs();
        let up = ax.next_up();
        if up.is_finite() {
            up - ax
        } else {
            // ax is f64::MAX — fall back to distance to next-down on the
            // positive side.
            ax - ax.next_down()
        }
    };
    MbValue::from_float(r)
}

// math.cbrt(x) — Py3.11+. Cube root with sign (cbrt(-8) == -2.0).
pub fn mb_math_cbrt(val: MbValue) -> MbValue {
    as_f64(val)
        .map(|f| MbValue::from_float(f.cbrt()))
        .unwrap_or(MbValue::none())
}

// math.exp2(x) — Py3.11+. 2**x; faster and more accurate than pow(2, x).
pub fn mb_math_exp2(val: MbValue) -> MbValue {
    as_f64(val)
        .map(|f| MbValue::from_float(f.exp2()))
        .unwrap_or(MbValue::none())
}

// math.fsum(iterable) — accurate floating-point sum using Shewchuk's
// algorithm (the same approach CPython uses). Maintains a list of
// non-overlapping partial sums whose exact sum equals the input.
pub fn mb_math_fsum(val: MbValue) -> MbValue {
    let items = match extract_seq_floats(val) {
        Some(v) => v,
        None => return MbValue::none(),
    };
    let mut partials: Vec<f64> = Vec::new();
    for x_in in items {
        let mut x = x_in;
        let mut i = 0;
        for j in 0..partials.len() {
            let mut y = partials[j];
            if x.abs() < y.abs() {
                std::mem::swap(&mut x, &mut y);
            }
            let hi = x + y;
            let lo = y - (hi - x);
            if lo != 0.0 {
                partials[i] = lo;
                i += 1;
            }
            x = hi;
        }
        partials.truncate(i);
        partials.push(x);
    }
    let total: f64 = partials.iter().sum();
    // CPython yields 0.0 (positive zero) for empty input; Rust sum starts
    // from 0.0 so this is mainly a guard against accidental -0.0 from
    // intermediate cancellation.
    let total = if total == 0.0 { 0.0 } else { total };
    MbValue::from_float(total)
}

/// Best-effort conversion of a list/tuple of numerics to `Vec<f64>`.
fn extract_seq_floats(val: MbValue) -> Option<Vec<f64>> {
    let items = match val.as_ptr() {
        Some(ptr) => unsafe {
            match &(*ptr).data {
                ObjData::List(ref lock) => lock.read().unwrap().to_vec(),
                ObjData::Tuple(items) => items.clone(),
                _ => return None,
            }
        },
        None => return None,
    };
    let mut out = Vec::with_capacity(items.len());
    for v in items {
        out.push(as_f64(v)?);
    }
    Some(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_math_sqrt() {
        let result = mb_math_sqrt(MbValue::from_float(16.0));
        assert_eq!(result.as_float(), Some(4.0));

        let result = mb_math_sqrt(MbValue::from_int(9));
        assert_eq!(result.as_float(), Some(3.0));
    }

    #[test]
    fn test_math_floor_ceil() {
        assert_eq!(mb_math_floor(MbValue::from_float(3.7)).as_int(), Some(3));
        assert_eq!(mb_math_ceil(MbValue::from_float(3.2)).as_int(), Some(4));
    }

    #[test]
    fn test_math_trig() {
        let pi = MbValue::from_float(std::f64::consts::PI);
        let sin_pi = mb_math_sin(pi).as_float().unwrap();
        assert!(sin_pi.abs() < 1e-10);

        let cos_0 = mb_math_cos(MbValue::from_float(0.0)).as_float().unwrap();
        assert!((cos_0 - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_math_gcd() {
        assert_eq!(
            mb_math_gcd(MbValue::from_int(12), MbValue::from_int(8)).as_int(),
            Some(4)
        );
        assert_eq!(
            mb_math_gcd(MbValue::from_int(7), MbValue::from_int(13)).as_int(),
            Some(1)
        );
    }

    #[test]
    fn test_math_factorial() {
        assert_eq!(mb_math_factorial(MbValue::from_int(0)).as_int(), Some(1));
        assert_eq!(mb_math_factorial(MbValue::from_int(5)).as_int(), Some(120));
        assert_eq!(
            mb_math_factorial(MbValue::from_int(10)).as_int(),
            Some(3628800)
        );
    }

    #[test]
    fn test_math_predicates() {
        assert_eq!(
            mb_math_isnan(MbValue::from_float(f64::NAN)).as_bool(),
            Some(true)
        );
        assert_eq!(
            mb_math_isnan(MbValue::from_float(1.0)).as_bool(),
            Some(false)
        );
        assert_eq!(
            mb_math_isinf(MbValue::from_float(f64::INFINITY)).as_bool(),
            Some(true)
        );
        assert_eq!(
            mb_math_isfinite(MbValue::from_float(42.0)).as_bool(),
            Some(true)
        );
    }

    #[test]
    fn test_math_pow() {
        let result = mb_math_pow(MbValue::from_int(2), MbValue::from_int(10));
        assert_eq!(result.as_float(), Some(1024.0));
    }

    #[test]
    fn test_math_trunc() {
        assert_eq!(mb_math_trunc(MbValue::from_float(3.9)).as_int(), Some(3));
        assert_eq!(mb_math_trunc(MbValue::from_float(-2.7)).as_int(), Some(-2));
    }

    #[test]
    fn test_math_fabs() {
        let r = mb_math_fabs(MbValue::from_float(-5.5)).as_float().unwrap();
        assert!((r - 5.5).abs() < 1e-10);
        let r2 = mb_math_fabs(MbValue::from_int(-3)).as_float().unwrap();
        assert!((r2 - 3.0).abs() < 1e-10);
    }

    #[test]
    fn test_math_inverse_trig() {
        let val = mb_math_asin(MbValue::from_float(1.0)).as_float().unwrap();
        assert!((val - std::f64::consts::FRAC_PI_2).abs() < 1e-10);

        let val = mb_math_acos(MbValue::from_float(1.0)).as_float().unwrap();
        assert!(val.abs() < 1e-10);

        let val = mb_math_atan(MbValue::from_float(0.0)).as_float().unwrap();
        assert!(val.abs() < 1e-10);
    }

    #[test]
    fn test_math_tan() {
        let val = mb_math_tan(MbValue::from_float(0.0)).as_float().unwrap();
        assert!(val.abs() < 1e-10);
    }

    #[test]
    fn test_math_exp_log() {
        let e1 = mb_math_exp(MbValue::from_float(0.0)).as_float().unwrap();
        assert!((e1 - 1.0).abs() < 1e-10);

        let ln_e = mb_math_log(MbValue::from_float(std::f64::consts::E))
            .as_float()
            .unwrap();
        assert!((ln_e - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_math_log2_log10() {
        let l2 = mb_math_log2(MbValue::from_float(8.0)).as_float().unwrap();
        assert!((l2 - 3.0).abs() < 1e-10);

        let l10 = mb_math_log10(MbValue::from_float(1000.0))
            .as_float()
            .unwrap();
        assert!((l10 - 3.0).abs() < 1e-10);
    }

    #[test]
    fn test_math_degrees_radians() {
        let deg = mb_math_degrees(MbValue::from_float(std::f64::consts::PI))
            .as_float()
            .unwrap();
        assert!((deg - 180.0).abs() < 1e-10);

        let rad = mb_math_radians(MbValue::from_float(180.0))
            .as_float()
            .unwrap();
        assert!((rad - std::f64::consts::PI).abs() < 1e-10);
    }

    #[test]
    fn test_math_atan2() {
        let r = mb_math_atan2(MbValue::from_float(1.0), MbValue::from_float(1.0))
            .as_float()
            .unwrap();
        assert!((r - std::f64::consts::FRAC_PI_4).abs() < 1e-10);
    }

    #[test]
    fn test_math_fmod() {
        let r = mb_math_fmod(MbValue::from_float(10.0), MbValue::from_float(3.0))
            .as_float()
            .unwrap();
        assert!((r - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_math_copysign() {
        let r = mb_math_copysign(MbValue::from_float(5.0), MbValue::from_float(-1.0))
            .as_float()
            .unwrap();
        assert!((r - (-5.0)).abs() < 1e-10);
    }

    #[test]
    fn test_math_hypot() {
        let r = mb_math_hypot(MbValue::from_float(3.0), MbValue::from_float(4.0))
            .as_float()
            .unwrap();
        assert!((r - 5.0).abs() < 1e-10);
    }

    #[test]
    fn test_math_comb() {
        assert_eq!(
            mb_math_comb(MbValue::from_int(5), MbValue::from_int(2)).as_int(),
            Some(10)
        );
        assert_eq!(
            mb_math_comb(MbValue::from_int(10), MbValue::from_int(0)).as_int(),
            Some(1)
        );
        assert_eq!(
            mb_math_comb(MbValue::from_int(6), MbValue::from_int(6)).as_int(),
            Some(1)
        );
    }

    #[test]
    fn test_math_isclose() {
        let r = mb_math_isclose(MbValue::from_float(1.0), MbValue::from_float(1.0 + 1e-10));
        assert_eq!(r.as_bool(), Some(true));

        let r2 = mb_math_isclose(MbValue::from_float(1.0), MbValue::from_float(2.0));
        assert_eq!(r2.as_bool(), Some(false));
    }

    #[test]
    fn test_math_factorial_edge_cases() {
        // CPython 3.12: factorial(-1) raises ValueError
        assert!(mb_math_factorial(MbValue::from_int(-1)).is_none());
        super::super::super::exception::mb_clear_exception();
        assert!(mb_math_factorial(MbValue::from_int(21)).is_none());
    }

    #[test]
    fn test_math_gcd_negative() {
        assert_eq!(
            mb_math_gcd(MbValue::from_int(-12), MbValue::from_int(8)).as_int(),
            Some(4)
        );
    }

    #[test]
    fn test_math_none_input() {
        assert!(mb_math_sqrt(MbValue::none()).is_none());
        assert!(mb_math_pow(MbValue::none(), MbValue::from_int(2)).is_none());
        assert!(mb_math_isfinite(MbValue::none()).as_bool().unwrap());
    }

    // -- Py3.12 conformance --

    #[test]
    fn test_py312_math_sqrt_of_4_is_2() {
        let r = mb_math_sqrt(MbValue::from_int(4));
        assert!((r.as_float().unwrap() - 2.0).abs() < 1e-12);
    }

    #[test]
    fn test_py312_math_sqrt_negative_raises_error() {
        // CPython 3.12: math.sqrt(-1) raises ValueError
        let r = mb_math_sqrt(MbValue::from_int(-1));
        assert!(r.is_none()); // Returns None after raising ValueError
        super::super::super::exception::mb_clear_exception();
    }

    #[test]
    fn test_py312_math_floor_negative() {
        let r = mb_math_floor(MbValue::from_float(-2.3));
        assert_eq!(r.as_int(), Some(-3));
    }

    #[test]
    fn test_py312_math_ceil_negative() {
        let r = mb_math_ceil(MbValue::from_float(-2.7));
        assert_eq!(r.as_int(), Some(-2));
    }

    #[test]
    fn test_py312_math_isnan_true() {
        let r = mb_math_isnan(MbValue::from_float(f64::NAN));
        assert_eq!(r.as_bool(), Some(true));
    }

    #[test]
    fn test_py312_math_isnan_false_for_normal() {
        let r = mb_math_isnan(MbValue::from_float(3.14));
        assert_eq!(r.as_bool(), Some(false));
    }

    #[test]
    fn test_py312_math_isinf_positive() {
        let r = mb_math_isinf(MbValue::from_float(f64::INFINITY));
        assert_eq!(r.as_bool(), Some(true));
    }

    #[test]
    fn test_py312_math_isfinite_for_zero() {
        let r = mb_math_isfinite(MbValue::from_float(0.0));
        assert_eq!(r.as_bool(), Some(true));
    }

    #[test]
    fn test_py312_math_log_base_e() {
        let r = mb_math_log(MbValue::from_float(std::f64::consts::E));
        assert!((r.as_float().unwrap() - 1.0).abs() < 1e-12);
    }

    #[test]
    fn test_py312_math_pi_constant() {
        let pi = std::f64::consts::PI;
        assert!((pi - 3.14159265).abs() < 1e-7);
    }
}
