use super::super::{
    closure::{mb_cell_compare_value, CellCompareValue},
    value::MbValue,
};

pub(super) fn cell_values_eq(a: MbValue, b: MbValue) -> Option<bool> {
    use CellCompareValue::{Empty, NotACell, Value};

    match (mb_cell_compare_value(a), mb_cell_compare_value(b)) {
        (NotACell, NotACell) => None,
        (Empty, Empty) => Some(true),
        (Empty, _) | (_, Empty) => Some(false),
        (Value(av), Value(bv)) => {
            if let (Some(ai), Some(bf)) = (av.as_int_pyint(), bv.as_float()) {
                return Some((ai as f64) == bf);
            }
            if let (Some(af), Some(bi)) = (av.as_float(), bv.as_int_pyint()) {
                return Some(af == (bi as f64));
            }
            if let (Some(ai), Some(bi)) = (av.as_int_pyint(), bv.as_int_pyint()) {
                return Some(ai == bi);
            }
            Some(super::mb_values_eq(av, bv))
        }
        (Value(av), NotACell) => Some(super::mb_values_eq(av, b)),
        (NotACell, Value(bv)) => Some(super::mb_values_eq(a, bv)),
    }
}

pub(super) fn cell_values_lt(a: MbValue, b: MbValue) -> Option<bool> {
    use CellCompareValue::{Empty, NotACell, Value};

    match (mb_cell_compare_value(a), mb_cell_compare_value(b)) {
        (NotACell, NotACell) => None,
        (Empty, Empty) => Some(false),
        (Empty, _) => Some(true),
        (_, Empty) => Some(false),
        (Value(av), Value(bv)) => {
            if let (Some(ai), Some(bf)) = (av.as_int_pyint(), bv.as_float()) {
                return Some((ai as f64) < bf);
            }
            if let (Some(af), Some(bi)) = (av.as_float(), bv.as_int_pyint()) {
                return Some(af < (bi as f64));
            }
            if let (Some(ai), Some(bi)) = (av.as_int_pyint(), bv.as_int_pyint()) {
                return Some(ai < bi);
            }
            Some(super::mb_values_lt(av, bv))
        }
        (Value(av), NotACell) => Some(super::mb_values_lt(av, b)),
        (NotACell, Value(bv)) => Some(super::mb_values_lt(a, bv)),
    }
}
