// Copyright (c) Aptos
// SPDX-License-Identifier: Apache-2.0


use move_core_types::effects::Op;
use aptos_consensus_types::common::{Payload, Round};
use aptos_crypto::{bls12381, HashValue};
use aptos_types::aggregate_signature::AggregateSignature;
use aptos_types::PeerId;
use aptos_types::validator_verifier::ValidatorVerifier;
use crate::common::Payload;
use anyhow::Context;


pub enum SignedNodeDigestError {
    WrongDigest,
    DuplicatedSignature,
}

#[derive(Deserialize, Serialize, Clone, Debug, PartialEq, Eq)]
pub struct NodeCertificate {
    digest: HashValue,
    multi_signature: AggregateSignature,
}

impl NodeCertificate {
    pub fn new(digest: HashValue, multi_signature: AggregateSignature) -> Self {
        Self {
            digest,
            multi_signature,
        }
    }

    pub fn digest(&self) -> &HashValue {
        &self.digest
    }

    pub fn verify(&self, validator: &ValidatorVerifier) -> anyhow::Result<()> {
        validator
            .verify_multi_signatures(&self.digest, &self.multi_signature)
            .context("Failed to verify ProofOfStore")
    }
}

pub struct Node {
    epoch: u64,
    round: u64,
    source: PeerId,
    consensus_payload: Payload,
    parents: Vec<HashValue>,
}


pub struct CertifiedNode {
    header: Node,
    certificate: NodeCertificate,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SignedNodeDigest {
    digest: HashValue,
    peer_id: PeerId,
    signature: bls12381::Signature,
}

impl SignedNodeDigest {
    pub fn new(digest: HashValue, peer_id: PeerId, signature: bls12381::Signature) -> Self {
        Self {
            digest,
            peer_id,
            signature,
        }
    }

    pub fn verify(&self, validator: &ValidatorVerifier) -> anyhow::Result<()> {
        Ok(validator.verify(self.peer_id, &self.digest, &self.signature)?)
    }

    pub fn digest(&self) -> HashValue {
        self.digest
    }

    pub fn peer_id(&self) -> PeerId {
        self.peer_id
    }

    pub fn signature(self) -> bls12381::Signature {
        self.signature
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CertifiedNodeAck {
    digest: HashValue,
    peer_id: PeerId,
}