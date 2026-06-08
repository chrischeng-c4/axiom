use super::super::rc::MbObject;
use super::super::value::MbValue;
/// colorsys module for Mamba (mamba-stdlib).
///
/// Pure-math color-space conversion functions mirroring CPython 3.12
/// Lib/colorsys.py. All inputs and outputs are floats in [0.0, 1.0]
/// (except YIQ i/q channels which may exceed that range).
use std::collections::HashMap;

macro_rules! dispatch_ternary {
    ($name:ident, $fn:ident) => {
        unsafe extern "C" fn $name(args_ptr: *const MbValue, nargs: usize) -> MbValue {
            let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
            $fn(
                a.get(0).copied().unwrap_or_else(MbValue::none),
                a.get(1).copied().unwrap_or_else(MbValue::none),
                a.get(2).copied().unwrap_or_else(MbValue::none),
            )
        }
    };
}

dispatch_ternary!(dispatch_rgb_to_hsv, mb_colorsys_rgb_to_hsv);
dispatch_ternary!(dispatch_hsv_to_rgb, mb_colorsys_hsv_to_rgb);
dispatch_ternary!(dispatch_rgb_to_hls, mb_colorsys_rgb_to_hls);
dispatch_ternary!(dispatch_hls_to_rgb, mb_colorsys_hls_to_rgb);
dispatch_ternary!(dispatch_rgb_to_yiq, mb_colorsys_rgb_to_yiq);
dispatch_ternary!(dispatch_yiq_to_rgb, mb_colorsys_yiq_to_rgb);

pub fn register() {
    let mut attrs = HashMap::new();
    let dispatchers: Vec<(&str, usize)> = vec![
        ("rgb_to_hsv", dispatch_rgb_to_hsv as usize),
        ("hsv_to_rgb", dispatch_hsv_to_rgb as usize),
        ("rgb_to_hls", dispatch_rgb_to_hls as usize),
        ("hls_to_rgb", dispatch_hls_to_rgb as usize),
        ("rgb_to_yiq", dispatch_rgb_to_yiq as usize),
        ("yiq_to_rgb", dispatch_yiq_to_rgb as usize),
    ];
    for (name, addr) in dispatchers {
        attrs.insert(name.to_string(), MbValue::from_func(addr));
        super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
            s.borrow_mut().insert(addr as u64);
        });
    }

    // Module-level constants (undocumented in CPython but part of the
    // public surface — used by HLS helpers).
    attrs.insert("ONE_SIXTH".to_string(), MbValue::from_float(1.0 / 6.0));
    attrs.insert("ONE_THIRD".to_string(), MbValue::from_float(1.0 / 3.0));
    attrs.insert("TWO_THIRD".to_string(), MbValue::from_float(2.0 / 3.0));

    super::register_module("colorsys", attrs);
}

// ── Helper ──

fn extract_float(val: MbValue) -> f64 {
    val.as_float().unwrap_or(0.0)
}

/// Pack three f64 components into a 3-element Tuple MbValue.
fn pack3(a: f64, b: f64, c: f64) -> MbValue {
    MbValue::from_ptr(MbObject::new_tuple(vec![
        MbValue::from_float(a),
        MbValue::from_float(b),
        MbValue::from_float(c),
    ]))
}

// ── Internal math helper for HLS ──

fn _v(m1: f64, m2: f64, hue: f64) -> f64 {
    let hue = hue % 1.0;
    // Ensure hue is positive after modulo (matches Python's behaviour)
    let hue = if hue < 0.0 { hue + 1.0 } else { hue };
    if hue < 1.0 / 6.0 {
        return m1 + (m2 - m1) * hue * 6.0;
    }
    if hue < 0.5 {
        return m2;
    }
    if hue < 2.0 / 3.0 {
        return m1 + (m2 - m1) * (2.0 / 3.0 - hue) * 6.0;
    }
    m1
}

// ── Six public conversion functions ──

// REQ: REQ-001
/// rgb_to_hsv(r, g, b) → (h, s, v)  — mirrors CPython colorsys.rgb_to_hsv.
pub fn mb_colorsys_rgb_to_hsv(r: MbValue, g: MbValue, b: MbValue) -> MbValue {
    let r = extract_float(r);
    let g = extract_float(g);
    let b = extract_float(b);

    let maxc = r.max(g).max(b);
    let minc = r.min(g).min(b);
    let rangec = maxc - minc;
    let v = maxc;

    if maxc == minc {
        return pack3(0.0, 0.0, v);
    }

    let s = rangec / maxc;
    let rc = (maxc - r) / rangec;
    let gc = (maxc - g) / rangec;
    let bc = (maxc - b) / rangec;

    let h = if r == maxc {
        bc - gc
    } else if g == maxc {
        2.0 + rc - bc
    } else {
        4.0 + gc - rc
    };

    let h = (h / 6.0).rem_euclid(1.0);
    pack3(h, s, v)
}

// REQ: REQ-002
/// hsv_to_rgb(h, s, v) → (r, g, b)  — mirrors CPython colorsys.hsv_to_rgb.
pub fn mb_colorsys_hsv_to_rgb(h: MbValue, s: MbValue, v: MbValue) -> MbValue {
    let h = extract_float(h);
    let s = extract_float(s);
    let v = extract_float(v);

    if s == 0.0 {
        return pack3(v, v, v);
    }

    let i = (h * 6.0).floor() as i64;
    let f = (h * 6.0) - i as f64;
    let p = v * (1.0 - s);
    let q = v * (1.0 - s * f);
    let t = v * (1.0 - s * (1.0 - f));

    let (r, g, b) = match i % 6 {
        0 => (v, t, p),
        1 => (q, v, p),
        2 => (p, v, t),
        3 => (p, q, v),
        4 => (t, p, v),
        _ => (v, p, q), // 5
    };

    pack3(r, g, b)
}

// REQ: REQ-003
/// rgb_to_hls(r, g, b) → (h, l, s)  — mirrors CPython colorsys.rgb_to_hls.
pub fn mb_colorsys_rgb_to_hls(r: MbValue, g: MbValue, b: MbValue) -> MbValue {
    let r = extract_float(r);
    let g = extract_float(g);
    let b = extract_float(b);

    let maxc = r.max(g).max(b);
    let minc = r.min(g).min(b);
    let sumc = maxc + minc;
    let rangec = maxc - minc;
    let l = sumc / 2.0;

    if maxc == minc {
        return pack3(0.0, l, 0.0);
    }

    let s = if l <= 0.5 {
        rangec / sumc
    } else {
        rangec / (2.0 - maxc - minc)
    };

    let rc = (maxc - r) / rangec;
    let gc = (maxc - g) / rangec;
    let bc = (maxc - b) / rangec;

    let h = if r == maxc {
        bc - gc
    } else if g == maxc {
        2.0 + rc - bc
    } else {
        4.0 + gc - rc
    };

    let h = (h / 6.0).rem_euclid(1.0);
    pack3(h, l, s)
}

// REQ: REQ-004
/// hls_to_rgb(h, l, s) → (r, g, b)  — mirrors CPython colorsys.hls_to_rgb.
pub fn mb_colorsys_hls_to_rgb(h: MbValue, l: MbValue, s: MbValue) -> MbValue {
    let h = extract_float(h);
    let l = extract_float(l);
    let s = extract_float(s);

    if s == 0.0 {
        return pack3(l, l, l);
    }

    let m2 = if l <= 0.5 {
        l * (1.0 + s)
    } else {
        l + s - (l * s)
    };
    let m1 = 2.0 * l - m2;

    pack3(
        _v(m1, m2, h + 1.0 / 3.0),
        _v(m1, m2, h),
        _v(m1, m2, h - 1.0 / 3.0),
    )
}

// REQ: REQ-005
/// rgb_to_yiq(r, g, b) → (y, i, q)  — mirrors CPython colorsys.rgb_to_yiq.
pub fn mb_colorsys_rgb_to_yiq(r: MbValue, g: MbValue, b: MbValue) -> MbValue {
    let r = extract_float(r);
    let g = extract_float(g);
    let b = extract_float(b);

    let y = 0.30 * r + 0.59 * g + 0.11 * b;
    let i = 0.74 * (r - y) - 0.27 * (b - y);
    let q = 0.48 * (r - y) + 0.41 * (b - y);
    pack3(y, i, q)
}

// REQ: REQ-006
/// yiq_to_rgb(y, i, q) → (r, g, b)  — mirrors CPython colorsys.yiq_to_rgb.
/// Note: CPython does NOT clamp outputs to [0, 1]; neither do we.
pub fn mb_colorsys_yiq_to_rgb(y: MbValue, i: MbValue, q: MbValue) -> MbValue {
    let y = extract_float(y);
    let i = extract_float(i);
    let q = extract_float(q);

    let r = y + 0.9468822170900693 * i + 0.6235565819861433 * q;
    let g = y - 0.27478764629897834 * i - 0.6356910791873801 * q;
    let b = y - 1.1085450346420322 * i + 1.7090069284064666 * q;
    pack3(r, g, b)
}

// ── Tests ──

#[cfg(test)]
mod tests {
    use super::super::super::rc::ObjData;
    use super::*;

    fn f(v: f64) -> MbValue {
        MbValue::from_float(v)
    }

    /// Unpack a Tuple MbValue into (f64, f64, f64).
    fn unpack3(val: MbValue) -> (f64, f64, f64) {
        unsafe {
            let ptr = val.as_ptr().expect("expected pointer");
            let ObjData::Tuple(ref items) = (*ptr).data else {
                panic!("expected Tuple, got something else");
            };
            (
                items[0].as_float().expect("item[0] not float"),
                items[1].as_float().expect("item[1] not float"),
                items[2].as_float().expect("item[2] not float"),
            )
        }
    }

    // REQ: REQ-001
    #[test]
    fn test_rgb_to_hsv_red() {
        let result = mb_colorsys_rgb_to_hsv(f(1.0), f(0.0), f(0.0));
        let (h, s, v) = unpack3(result);
        assert!((h - 0.0).abs() < 1e-9, "h={h}");
        assert!((s - 1.0).abs() < 1e-9, "s={s}");
        assert!((v - 1.0).abs() < 1e-9, "v={v}");
    }

    // REQ: REQ-001, REQ-002
    #[test]
    fn test_rgb_hsv_roundtrip() {
        let (r0, g0, b0) = (0.3_f64, 0.6_f64, 0.2_f64);
        let hsv = mb_colorsys_rgb_to_hsv(f(r0), f(g0), f(b0));
        let (h, s, v) = unpack3(hsv);
        let rgb = mb_colorsys_hsv_to_rgb(f(h), f(s), f(v));
        let (r1, g1, b1) = unpack3(rgb);
        assert!((r1 - r0).abs() < 1e-9, "r diff={}", (r1 - r0).abs());
        assert!((g1 - g0).abs() < 1e-9, "g diff={}", (g1 - g0).abs());
        assert!((b1 - b0).abs() < 1e-9, "b diff={}", (b1 - b0).abs());
    }

    // REQ: REQ-003, REQ-004
    #[test]
    fn test_rgb_hls_roundtrip() {
        let (r0, g0, b0) = (0.3_f64, 0.6_f64, 0.2_f64);
        let hls = mb_colorsys_rgb_to_hls(f(r0), f(g0), f(b0));
        let (h, l, s) = unpack3(hls);
        let rgb = mb_colorsys_hls_to_rgb(f(h), f(l), f(s));
        let (r1, g1, b1) = unpack3(rgb);
        assert!((r1 - r0).abs() < 1e-9, "r diff={}", (r1 - r0).abs());
        assert!((g1 - g0).abs() < 1e-9, "g diff={}", (g1 - g0).abs());
        assert!((b1 - b0).abs() < 1e-9, "b diff={}", (b1 - b0).abs());
    }

    // REQ: REQ-005, REQ-006
    #[test]
    fn test_rgb_yiq_roundtrip() {
        let (r0, g0, b0) = (0.3_f64, 0.6_f64, 0.2_f64);
        let yiq = mb_colorsys_rgb_to_yiq(f(r0), f(g0), f(b0));
        let (y, i, q) = unpack3(yiq);
        let rgb = mb_colorsys_yiq_to_rgb(f(y), f(i), f(q));
        let (r1, g1, b1) = unpack3(rgb);
        assert!((r1 - r0).abs() < 1e-9, "r diff={}", (r1 - r0).abs());
        assert!((g1 - g0).abs() < 1e-9, "g diff={}", (g1 - g0).abs());
        assert!((b1 - b0).abs() < 1e-9, "b diff={}", (b1 - b0).abs());
    }

    // REQ: REQ-005, REQ-006
    #[test]
    fn test_yiq_black_and_white() {
        // Black: (0,0,0) → YIQ=(0,0,0)
        let (y, i, q) = unpack3(mb_colorsys_rgb_to_yiq(f(0.0), f(0.0), f(0.0)));
        assert!((y - 0.0).abs() < 1e-9);
        assert!((i - 0.0).abs() < 1e-9);
        assert!((q - 0.0).abs() < 1e-9);
        // White: (1,1,1) → Y=1, I≈0, Q≈0
        let (y, i, q) = unpack3(mb_colorsys_rgb_to_yiq(f(1.0), f(1.0), f(1.0)));
        assert!((y - 1.0).abs() < 1e-9, "y={y}");
        assert!(i.abs() < 1e-9, "i={i}");
        assert!(q.abs() < 1e-9, "q={q}");
    }

    // REQ: REQ-001 (registration smoke-test)
    #[test]
    fn test_register_installs_module() {
        // Calling register() must not panic.
        // Because MODULES is thread_local and tests may share the same thread,
        // we just verify the call completes and the module name is queryable.
        register();
        use super::super::super::module::MODULES;
        let present = MODULES.with(|m| m.borrow().contains_key("colorsys"));
        assert!(
            present,
            "colorsys module should be in the registry after register()"
        );
    }
}
