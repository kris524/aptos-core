// Copyright (c) Aptos
// SPDX-License-Identifier: Apache-2.0

///! MoveVM and Session wrapped, to make sure Aptos natives and extensions are always installed and
///! taken care of after session finish.
use crate::{
    access_path_cache::AccessPathCache, aptos_vm_impl::convert_changeset_and_events_cached,
    natives::aptos_natives, transaction_metadata::TransactionMetadata,
};
use aptos_crypto::{hash::CryptoHash, HashValue};
use aptos_crypto_derive::{BCSCryptoHash, CryptoHasher};
use aptos_types::{
    block_metadata::BlockMetadata,
    transaction::{ChangeSet, SignatureCheckedTransaction},
};
use move_binary_format::errors::VMResult;
use move_core_types::{
    account_address::AccountAddress,
    effects::{ChangeSet as MoveChangeSet, Event as MoveEvent},
    resolver::MoveResolver,
    vm_status::VMStatus,
};
use move_vm_runtime::{
    move_vm::MoveVM, native_functions::NativeContextExtensions, session::Session,
};
use serde::{Deserialize, Serialize};
use std::{
    convert::TryInto,
    ops::{Deref, DerefMut},
};

pub struct MoveVmExt {
    inner: MoveVM,
}

impl MoveVmExt {
    pub fn new() -> VMResult<Self> {
        Ok(Self {
            inner: MoveVM::new(aptos_natives())?,
        })
    }

    pub fn new_session<'r, S: MoveResolver>(
        &self,
        remote: &'r S,
        _session_id: SessionId,
    ) -> SessionExt<'r, '_, S> {
        // TODO: install table extension
        let extensions = NativeContextExtensions::default();

        SessionExt {
            inner: self.inner.new_session_with_extensions(remote, extensions),
        }
    }
}

impl Deref for MoveVmExt {
    type Target = MoveVM;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

#[derive(BCSCryptoHash, CryptoHasher, Deserialize, Serialize)]
pub enum SessionId {
    Txn {
        sender: AccountAddress,
        sequence_number: u64,
    },
    BlockMeta {
        // block id
        id: HashValue,
    },
    Genesis {
        // id to identify this specific genesis build
        id: HashValue,
    },
    // For those runs that are not a transaction and the output of which won't be committed.
    Void,
}

impl SessionId {
    pub fn txn(txn: &SignatureCheckedTransaction) -> Self {
        Self::Txn {
            sender: txn.sender(),
            sequence_number: txn.sequence_number(),
        }
    }

    pub fn txn_meta(txn_data: &TransactionMetadata) -> Self {
        Self::Txn {
            sender: txn_data.sender,
            sequence_number: txn_data.sequence_number,
        }
    }

    pub fn genesis(id: HashValue) -> Self {
        Self::Genesis { id }
    }

    pub fn block_meta(block_meta: &BlockMetadata) -> Self {
        Self::BlockMeta {
            id: block_meta.id(),
        }
    }

    pub fn void() -> Self {
        Self::Void
    }

    pub fn as_uuid(&self) -> u128 {
        u128::from_be_bytes(
            self.hash().as_ref()[..16]
                .try_into()
                .expect("Slice to array conversion failed."),
        )
    }
}

pub struct SessionExt<'r, 'l, S> {
    inner: Session<'r, 'l, S>,
}

impl<'r, 'l, S> SessionExt<'r, 'l, S>
where
    S: MoveResolver,
{
    pub fn finish(self) -> VMResult<SessionOutput> {
        let (change_set, events, extensions) = self.inner.finish_with_extensions()?;
        Ok(SessionOutput {
            change_set,
            events,
            extensions,
        })
    }
}

impl<'r, 'l, S> Deref for SessionExt<'r, 'l, S> {
    type Target = Session<'r, 'l, S>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<'r, 'l, S> DerefMut for SessionExt<'r, 'l, S> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

pub struct SessionOutput {
    pub change_set: MoveChangeSet,
    pub events: Vec<MoveEvent>,
    pub extensions: NativeContextExtensions,
}

impl SessionOutput {
    pub fn into_change_set<C: AccessPathCache>(
        self,
        ap_cache: &mut C,
    ) -> Result<ChangeSet, VMStatus> {
        // TODO: consider table change set from the table extension
        convert_changeset_and_events_cached(ap_cache, self.change_set, self.events)
            .map(|(write_set, events)| ChangeSet::new(write_set, events))
    }

    pub fn unpack(self) -> (MoveChangeSet, Vec<MoveEvent>, NativeContextExtensions) {
        (self.change_set, self.events, self.extensions)
    }
}
