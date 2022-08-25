//! This module contains [`Domain`](`crate::domain::Domain`) structure
//! and related implementations and trait implementations.
//!
//! Note that the Genesis domain and account have a temporary
//! privileged position, and permission validation is turned off for
//! the Genesis block.
#![allow(clippy::std_instead_of_alloc)]

#[cfg(not(feature = "std"))]
use alloc::{alloc::alloc, boxed::Box, format, string::String, vec::Vec};
use core::str::FromStr;
#[cfg(feature = "std")]
use std::alloc::alloc;

pub use crate::ipfs::IpfsPath;
use derive_more::{Display, FromStr};
use getset::{Getters, MutGetters};
use iroha_crypto::PublicKey;
use iroha_data_model_derive::IdOrdEqHash;
use iroha_ffi::{IntoFfi, TryFromReprC};
use iroha_schema::IntoSchema;
use parity_scale_codec::{Decode, Encode};
use serde::{Deserialize, Serialize};

use crate::{
    account::{genesis::Genesis as _, Account, AccountsMap},
    asset::AssetDefinitionsMap,
    ffi::ffi_item,
    metadata::Metadata,
    prelude::{AssetDefinition, AssetDefinitionEntry},
    HasMetadata, Identifiable, Name, Registered,
};

/// The domain name of the genesis domain.
///
/// The genesis domain should only contain the genesis account.
pub const GENESIS_DOMAIN_NAME: &str = "genesis";

/// Genesis domain. It will contain only one `genesis` account.
#[derive(Debug, Decode, Encode, Deserialize, Serialize, IntoSchema)]
pub struct GenesisDomain {
    genesis_key: PublicKey,
}

impl GenesisDomain {
    /// Returns `GenesisDomain`.
    #[inline]
    #[must_use]
    pub const fn new(genesis_key: PublicKey) -> Self {
        Self { genesis_key }
    }
}

#[cfg(feature = "mutable_api")]
impl From<GenesisDomain> for Domain {
    fn from(domain: GenesisDomain) -> Self {
        #[cfg(not(feature = "std"))]
        use alloc::collections::btree_map;
        #[cfg(feature = "std")]
        use std::collections::btree_map;

        #[allow(clippy::expect_used)]
        Self {
            id: Id::from_str(GENESIS_DOMAIN_NAME).expect("Valid"),
            accounts: core::iter::once((
                <Account as Identifiable>::Id::genesis(),
                crate::account::genesis::GenesisAccount::new(domain.genesis_key).into(),
            ))
            .collect(),
            asset_definitions: btree_map::BTreeMap::default(),
            metadata: Metadata::default(),
            logo: None,
        }
    }
}

ffi_item! {
    /// Builder which can be submitted in a transaction to create a new [`Domain`]
    #[derive(
        Debug,
        Display,
        Clone,
        IdOrdEqHash,
        Decode,
        Encode,
        Deserialize,
        Serialize,
        IntoFfi,
        TryFromReprC,
        IntoSchema,
    )]
    #[id(type = "<Domain as Identifiable>::Id")]
    #[display(fmt = "[{id}]")]
    pub struct NewDomain {
        /// The identification associated with the domain builder.
        id: <Domain as Identifiable>::Id,
        /// The (IPFS) link to the logo of this domain.
        logo: Option<IpfsPath>,
        /// Metadata associated with the domain builder.
        metadata: Metadata,
    }
}

#[cfg(feature = "mutable_api")]
impl crate::Registrable for NewDomain {
    type Target = Domain;

    #[must_use]
    #[inline]
    fn build(self) -> Self::Target {
        Self::Target {
            id: self.id,
            accounts: AccountsMap::default(),
            asset_definitions: AssetDefinitionsMap::default(),
            metadata: self.metadata,
            logo: self.logo,
        }
    }
}

impl HasMetadata for NewDomain {
    #[inline]
    fn metadata(&self) -> &crate::metadata::Metadata {
        &self.metadata
    }
}

#[cfg_attr(
    all(feature = "ffi_export", not(feature = "ffi_import")),
    iroha_ffi::ffi_export
)]
#[cfg_attr(feature = "ffi_import", iroha_ffi::ffi_import)]
impl NewDomain {
    /// Create a [`NewDomain`], reserved for internal use.
    #[must_use]
    fn new(id: <Domain as Identifiable>::Id) -> Self {
        Self {
            id,
            logo: None,
            metadata: Metadata::default(),
        }
    }

    /// Identification
    pub(crate) fn id(&self) -> &<Domain as Identifiable>::Id {
        &self.id
    }

    /// Add [`logo`](IpfsPath) to the domain replacing previously defined value
    #[must_use]
    pub fn with_logo(mut self, logo: IpfsPath) -> Self {
        self.logo = Some(logo);
        self
    }

    /// Add [`Metadata`] to the domain replacing previously defined value
    #[must_use]
    pub fn with_metadata(mut self, metadata: Metadata) -> Self {
        self.metadata = metadata;
        self
    }
}

ffi_item! {
    /// Named group of [`Account`] and [`Asset`](`crate::asset::Asset`) entities.
    #[derive(
        Debug,
        Display,
        Clone,
        IdOrdEqHash,
        Getters,
        MutGetters,
        Decode,
        Encode,
        Deserialize,
        Serialize,
        IntoFfi,
        TryFromReprC,
        IntoSchema,
    )]
    #[cfg_attr(all(feature = "ffi_export", not(feature = "ffi_import")), iroha_ffi::ffi_export)]
    #[cfg_attr(feature = "ffi_import", iroha_ffi::ffi_import)]
    #[allow(clippy::multiple_inherent_impl)]
    #[display(fmt = "[{id}]")]
    #[id(type = "Id")]
    pub struct Domain {
        /// Identification of this [`Domain`].
        id: <Self as Identifiable>::Id,
        /// [`Account`]s of the domain.
        accounts: AccountsMap,
        /// [`Asset`](AssetDefinition)s defined of the `Domain`.
        asset_definitions: AssetDefinitionsMap,
        /// IPFS link to the `Domain` logo
        // FIXME: Getter implemented manually because `getset`
        // returns &Option<T> when it should return Option<&T>
        logo: Option<IpfsPath>,
        /// [`Metadata`] of this `Domain` as a key-value store.
        #[getset(get = "pub")]
        #[cfg_attr(feature = "mutable_api", getset(get_mut = "pub"))]
        metadata: Metadata,
    }
}

impl HasMetadata for Domain {
    #[inline]
    fn metadata(&self) -> &crate::metadata::Metadata {
        &self.metadata
    }
}

impl Registered for Domain {
    type With = NewDomain;
}

#[cfg_attr(
    all(feature = "ffi_export", not(feature = "ffi_import")),
    iroha_ffi::ffi_export
)]
#[cfg_attr(feature = "ffi_import", iroha_ffi::ffi_import)]
impl Domain {
    /// Construct builder for [`Domain`] identifiable by [`Id`].
    pub fn new(id: <Self as Identifiable>::Id) -> <Self as Registered>::With {
        <Self as Registered>::With::new(id)
    }

    /// IPFS link to the `Domain` logo
    pub fn logo(&self) -> Option<&IpfsPath> {
        self.logo.as_ref()
    }

    /// Return a reference to the [`Account`] corresponding to the account id.
    #[inline]
    pub fn account(&self, account_id: &<Account as Identifiable>::Id) -> Option<&Account> {
        self.accounts.get(account_id)
    }

    /// Return a reference to the asset definition corresponding to the asset definition id
    #[inline]
    pub fn asset_definition(
        &self,
        asset_definition_id: &<AssetDefinition as Identifiable>::Id,
    ) -> Option<&AssetDefinitionEntry> {
        self.asset_definitions.get(asset_definition_id)
    }

    /// Get an iterator over [`Account`]s of the `Domain`
    #[inline]
    pub fn accounts(&self) -> impl ExactSizeIterator<Item = &Account> {
        self.accounts.values()
    }

    /// Return `true` if the `Domain` contains [`Account`]
    #[inline]
    pub fn contains_account(&self, account_id: &<Account as Identifiable>::Id) -> bool {
        self.accounts.contains_key(account_id)
    }

    /// Get an iterator over asset definitions of the `Domain`
    #[inline]
    pub fn asset_definitions(&self) -> impl ExactSizeIterator<Item = &AssetDefinitionEntry> {
        self.asset_definitions.values()
    }
}

#[cfg(feature = "mutable_api")]
impl Domain {
    /// Return a mutable reference to the [`Account`] corresponding to the account id.
    #[inline]
    pub fn account_mut(
        &mut self,
        account_id: &<Account as Identifiable>::Id,
    ) -> Option<&mut Account> {
        self.accounts.get_mut(account_id)
    }

    /// Add [`Account`] into the [`Domain`] returning previous account stored under the same id
    #[inline]
    pub fn add_account(&mut self, account: Account) -> Option<Account> {
        self.accounts.insert(account.id().clone(), account)
    }

    /// Remove account from the [`Domain`] and return it
    #[inline]
    pub fn remove_account(
        &mut self,
        account_id: &<Account as Identifiable>::Id,
    ) -> Option<Account> {
        self.accounts.remove(account_id)
    }

    /// Get a mutable iterator over accounts of the domain
    #[inline]
    pub fn accounts_mut(&mut self) -> impl ExactSizeIterator<Item = &mut Account> {
        self.accounts.values_mut()
    }

    /// Get a mutable iterator over asset definitions of the [`Domain`]
    #[inline]
    pub fn asset_definition_mut(
        &mut self,
        asset_definition_id: &<AssetDefinition as Identifiable>::Id,
    ) -> Option<&mut AssetDefinitionEntry> {
        self.asset_definitions.get_mut(asset_definition_id)
    }

    /// Add asset definition into the [`Domain`] returning previous
    /// asset definition stored under the same id
    #[inline]
    pub fn add_asset_definition(
        &mut self,
        asset_definition: AssetDefinition,
        registered_by: <Account as Identifiable>::Id,
    ) -> Option<AssetDefinitionEntry> {
        let asset_definition = AssetDefinitionEntry::new(asset_definition, registered_by);

        self.asset_definitions
            .insert(asset_definition.definition().id().clone(), asset_definition)
    }

    /// Remove asset definition from the [`Domain`] and return it
    #[inline]
    pub fn remove_asset_definition(
        &mut self,
        asset_definition_id: &<AssetDefinition as Identifiable>::Id,
    ) -> Option<AssetDefinitionEntry> {
        self.asset_definitions.remove(asset_definition_id)
    }
}

impl FromIterator<Domain> for crate::Value {
    fn from_iter<T: IntoIterator<Item = Domain>>(iter: T) -> Self {
        iter.into_iter()
            .map(Into::into)
            .collect::<Vec<Self>>()
            .into()
    }
}

/// Identification of a [`Domain`].
#[derive(
    Debug,
    Display,
    FromStr,
    Clone,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    Decode,
    Encode,
    Deserialize,
    Serialize,
    IntoFfi,
    TryFromReprC,
    IntoSchema,
)]
#[display(fmt = "{name}")]
pub struct Id {
    /// [`Name`] unique to a [`Domain`] e.g. company name
    pub name: Name,
}

impl Id {
    /// Construct [`Id`] if the given domain `name` is valid.
    ///
    /// # Errors
    /// Fails if any sub-construction fails
    #[inline]
    pub const fn new(name: Name) -> Self {
        Self { name }
    }
}

/// The prelude re-exports most commonly used traits, structs and macros from this crate.
pub mod prelude {
    pub use super::{Domain, GenesisDomain, Id as DomainId, GENESIS_DOMAIN_NAME};
}
