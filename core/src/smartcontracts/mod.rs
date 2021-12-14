//! Smartcontract runtime.
//!
//! Currently only Iroha instructions are supported. If you want to
//! register another ISI, you should implement [`Execute`]. For
//! queries, implement [`ValidQuery`]. Expressions should implement
//! [`Evaluate`] and make sure that they are free of side-effects.

pub mod isi;
pub mod wasm;

use iroha_data_model::prelude::*;
pub use isi::*;

use super::wsv::WorldStateView;
use crate::wsv::WorldTrait;

/// Trait implementations provide actions to apply changes to [`WorldStateView`].
#[allow(clippy::missing_errors_doc)]
pub trait Execute<W: WorldTrait> {
    /// Error type returned by execute function
    type Error: std::error::Error;
    /// Difference between [`WorldStateView`] before and after execution of [`Self`].
    type Diff: Into<Vec<DataEvent>>;

    /// Apply actions to `wsv` on behalf of `authority`.
    fn execute(
        self,
        authority: AccountId,
        wsv: &WorldStateView<W>,
    ) -> Result<Self::Diff, Self::Error>;
}

/// Calculate the result of the expression without mutating the state.
#[allow(clippy::missing_errors_doc)]
pub trait Evaluate<W: WorldTrait> {
    /// The resulting type of the expression.
    type Value;
    /// Error type returned if the evaluation fails. Typically just [`isi::error::Error`].
    type Error: std::error::Error;

    /// Calculates result.
    fn evaluate(
        &self,
        wsv: &WorldStateView<W>,
        context: &Context,
    ) -> Result<Self::Value, Self::Error>;
}

/// Trait implemented by all Iroha Queries.
#[allow(clippy::missing_errors_doc)]
pub trait ValidQuery<W: WorldTrait>: Query {
    /// Execute query on the [`WorldStateView`].
    /// Should not mutate [`WorldStateView`]!
    ///
    /// Returns Ok(QueryResult) if succeeded and Err(String) if failed.
    fn execute(&self, wsv: &WorldStateView<W>) -> eyre::Result<Self::Output, query::Error>;

    /// Executes query and maps it into value
    fn execute_into_value(&self, wsv: &WorldStateView<W>) -> eyre::Result<Value, query::Error> {
        self.execute(wsv).map(Into::into)
    }
}
