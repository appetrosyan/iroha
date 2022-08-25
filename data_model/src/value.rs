//! The Value-oriented logic.
use super::*;

/// Create a [`Vec`] containing the arguments, which should satisfy `Into<Value>` bound.
///
/// Syntax is the same as in [`vec`](macro@vec)
#[macro_export]
macro_rules! val_vec {
    () => { Vec::new() };
    ($elem:expr; $n:expr) => { vec![$crate::value::Value::from($elem); $n] };
    ($($x:expr),+ $(,)?) => { vec![$($crate::value::Value::from($x),)+] };
}

/// Sized container for all possible values.
#[derive(
    Debug,
    Clone,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Decode,
    Encode,
    Deserialize,
    Serialize,
    FromVariant,
    IntoFfi,
    TryFromReprC,
    IntoSchema,
)]
#[allow(clippy::enum_variant_names)]
#[repr(u8)]
pub enum Value {
    /// [`u32`] integer.
    U32(u32),
    /// [`u128`] integer.
    U128(u128),
    /// [`bool`] value.
    Bool(bool),
    /// [`String`] value.
    String(String),
    /// [`Name`] value.
    Name(Name),
    /// [`fixed::Fixed`] value
    Fixed(fixed::Fixed),
    /// [`Vec`] of `Value`.
    Vec(
        #[skip_from]
        #[skip_try_from]
        Vec<Value>,
    ),
    /// Recursive inclusion of LimitedMetadata,
    LimitedMetadata(metadata::Metadata),
    /// `Id` of `Asset`, `Account`, etc.
    Id(IdBox),
    /// `impl Identifiable` as in `Asset`, `Account` etc.
    Identifiable(IdentifiableBox),
    /// [`PublicKey`].
    PublicKey(PublicKey),
    /// Iroha [`Parameter`] variant.
    Parameter(Parameter),
    /// Signature check condition.
    SignatureCheckCondition(SignatureCheckCondition),
    /// Committed or rejected transactions
    TransactionValue(TransactionValue),
    /// Transaction Query
    TransactionQueryResult(TransactionQueryResult),
    /// [`PermissionToken`].
    PermissionToken(PermissionToken),
    /// [`struct@Hash`]
    Hash(Hash),
    /// Block
    Block(block_value::BlockValueWrapper),
    /// Block headers
    BlockHeader(BlockHeaderValue),
}

#[allow(clippy::len_without_is_empty)]
impl Value {
    /// Number of underneath expressions.
    pub fn len(&self) -> usize {
        use Value::*;

        match self {
            U32(_)
            | U128(_)
            | Id(_)
            | PublicKey(_)
            | Bool(_)
            | Parameter(_)
            | Identifiable(_)
            | String(_)
            | Name(_)
            | Fixed(_)
            | TransactionValue(_)
            | TransactionQueryResult(_)
            | PermissionToken(_)
            | Hash(_)
            | Block(_)
            | BlockHeader(_) => 1_usize,
            Vec(v) => v.iter().map(Self::len).sum::<usize>() + 1_usize,
            LimitedMetadata(data) => data.nested_len() + 1_usize,
            SignatureCheckCondition(s) => s.0.len(),
        }
    }
}

impl fmt::Display for Value {
    // TODO: Maybe derive
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::U32(v) => fmt::Display::fmt(&v, f),
            Value::U128(v) => fmt::Display::fmt(&v, f),
            Value::Bool(v) => fmt::Display::fmt(&v, f),
            Value::String(v) => fmt::Display::fmt(&v, f),
            Value::Name(v) => fmt::Display::fmt(&v, f),
            Value::Fixed(v) => fmt::Display::fmt(&v, f),
            #[allow(clippy::use_debug)]
            Value::Vec(v) => {
                // TODO: Remove so we can derive.
                let list_of_display: Vec<_> = v.iter().map(ToString::to_string).collect();
                // this prints with quotation marks, which is fine 90%
                // of the time, and helps delineate where a display of
                // one value stops and another one begins.
                write!(f, "{:?}", list_of_display)
            }
            Value::LimitedMetadata(v) => fmt::Display::fmt(&v, f),
            Value::Id(v) => fmt::Display::fmt(&v, f),
            Value::Identifiable(v) => fmt::Display::fmt(&v, f),
            Value::PublicKey(v) => fmt::Display::fmt(&v, f),
            Value::Parameter(v) => fmt::Display::fmt(&v, f),
            Value::SignatureCheckCondition(v) => fmt::Display::fmt(&v, f),
            Value::TransactionValue(_) => write!(f, "TransactionValue"),
            Value::TransactionQueryResult(_) => write!(f, "TransactionQueryResult"),
            Value::PermissionToken(v) => fmt::Display::fmt(&v, f),
            Value::Hash(v) => fmt::Display::fmt(&v, f),
            Value::Block(v) => fmt::Display::fmt(&**v, f),
            Value::BlockHeader(v) => fmt::Display::fmt(&v, f),
        }
    }
}

pub mod prelude {
    pub use super::Value;
}
