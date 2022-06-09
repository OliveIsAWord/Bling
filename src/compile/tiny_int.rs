use num_bigint::BigInt;
use num_traits::{Signed, Zero};
use std::convert::{From, TryFrom};
use std::{fmt, ops};

#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub enum TinyInt {
    Inline(isize),
    Heap(BigInt),
}
use TinyInt::{Heap, Inline};

impl TinyInt {
    pub fn is_zero(&self) -> bool {
        match self {
            Inline(x) => x.is_zero(),
            Heap(h) => h.is_zero(),
        }
    }
    pub fn is_negative(&self) -> bool {
        match self {
            Inline(x) => x.is_negative(),
            Heap(h) => h.is_negative(),
        }
    }
    pub const fn zero() -> Self {
        Inline(0)
    }
    pub fn checked_div(self, rhs: &Self) -> Option<Self> {
        let checked_isize_div = |x: isize, y: isize| {
            x.checked_div(y)
                .map(Inline)
                .or_else(|| (y == -1).then(|| Heap(-BigInt::from(x))))
        };
        match (self, rhs) {
            (Inline(x), &Inline(y)) => checked_isize_div(x, y),
            (Heap(x), &Inline(y)) => (y != 0).then(|| Heap(x / y)),
            (Inline(x), Heap(y)) => {
                if let Ok(d) = y.try_into() {
                    checked_isize_div(x, d)
                } else if y.try_into() == Ok(isize::MAX as usize + 1) && x == isize::MIN {
                    Some(Inline(-1))
                } else {
                    Some(Inline(0))
                }
            }
            (Heap(x), Heap(y)) => x.checked_div(y).map(Heap),
        }
    }
}

macro_rules! impl_op {
    ($op_trait:path, $op:ident, $checked_op:ident) => {
        impl $op_trait for TinyInt {
            type Output = Self;
            fn $op(self, rhs: Self) -> Self {
                match (self, rhs) {
                    (Inline(x), Inline(y)) => x
                        .$checked_op(y)
                        .map_or_else(|| Heap(BigInt::from(x).$op(y)), Inline),
                    (Heap(h), Inline(x)) => Heap(h.$op(x)),
                    (Inline(x), Heap(h)) => Heap(x.$op(h)),
                    (Heap(h1), Heap(h2)) => Heap(h1.$op(h2)),
                }
            }
        }
    };
}

impl_op! {ops::Add, add, checked_add}
impl_op! {ops::Sub, sub, checked_sub}
impl_op! {ops::Mul, mul, checked_mul}
//impl_op! {ops::Div, div}
impl_op! {ops::Rem, rem, checked_rem}

impl ops::Neg for TinyInt {
    type Output = Self;
    fn neg(self) -> Self {
        match self {
            Inline(x) => x.checked_neg().map_or(Heap(-BigInt::from(x)), Inline),
            Heap(h) => Heap(h.neg()),
        }
    }
}

impl fmt::Display for TinyInt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Inline(x) => write!(f, "{}", x),
            Heap(h) => write!(f, "{}", h),
        }
    }
}

impl From<isize> for TinyInt {
    fn from(x: isize) -> Self {
        Inline(x)
    }
}
impl From<usize> for TinyInt {
    fn from(x: usize) -> Self {
        x.try_into().map_or_else(|_| Heap(BigInt::from(x)), Inline)
    }
}
impl From<BigInt> for TinyInt {
    fn from(x: BigInt) -> Self {
        match x.try_into() {
            Ok(i) => Inline(i),
            Err(e) => Heap(e.into_original()),
        }
    }
}
impl TryFrom<TinyInt> for usize {
    type Error = ();
    fn try_from(v: TinyInt) -> Result<Self, ()> {
        match v {
            Inline(x) => x.try_into().map_err(|_| ()),
            Heap(h) => h.try_into().map_err(|_| ()),
        }
    }
}
