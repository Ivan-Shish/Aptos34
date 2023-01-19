// Copyright (c) Aptos
// SPDX-License-Identifier: Apache-2.0


use std::sync::Arc;
use aptos_crypto::{bls12381, CryptoMaterialError, HashValue};
use aptos_types::aggregate_signature::AggregateSignature;
use aptos_types::PeerId;
use aptos_types::validator_verifier::ValidatorVerifier;
use crate::common::Payload;
use anyhow::Context;
use serde::{Deserialize, Serialize};
use aptos_crypto_derive::{BCSCryptoHash, CryptoHasher};
use aptos_types::validator_signer::ValidatorSigner;


pub enum SignedNodeDigestError {
    WrongDigest,
    DuplicatedSignature,
}


#[derive(
Clone, Debug, Deserialize, Serialize, CryptoHasher, BCSCryptoHash, PartialEq, Eq, Hash,
)]
pub struct SignedNodeDigestInfo {
    digest: HashValue,
}

impl SignedNodeDigestInfo {
    pub fn new(digest: HashValue) -> Self {
        Self {
            digest
        }
    }

    pub fn digest(&self) -> HashValue {
        self.digest
    }
}

#[allow(dead_code)]
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SignedNodeDigest {
    signed_node_digest_info: SignedNodeDigestInfo,
    peer_id: PeerId,
    signature: bls12381::Signature,
}

impl SignedNodeDigest {
    pub fn new(digest: HashValue, validator_signer: Arc<ValidatorSigner>) -> Result<Self, CryptoMaterialError> {
        let info = SignedNodeDigestInfo::new(digest);
        let signature = validator_signer.sign(&info)?;

        Ok(Self {
            signed_node_digest_info: SignedNodeDigestInfo::new(digest),
            peer_id: validator_signer.author(),
            signature,
        })
    }

    pub fn verify(&self, validator: &ValidatorVerifier) -> anyhow::Result<()> {
        Ok(validator.verify(self.peer_id, &self.signed_node_digest_info, &self.signature)?)
    }

    pub fn digest(&self) -> HashValue {
        self.signed_node_digest_info.digest
    }

    pub fn info(&self) -> &SignedNodeDigestInfo {
        &self.signed_node_digest_info
    }

    pub fn peer_id(&self) -> PeerId {
        self.peer_id
    }

    pub fn signature(self) -> bls12381::Signature {
        self.signature
    }
}

#[allow(dead_code)]
#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct NodeCertificate {
    signed_node_digest_info: SignedNodeDigestInfo,
    multi_signature: AggregateSignature,
}

impl NodeCertificate {
    pub fn new(signed_node_digest_info: SignedNodeDigestInfo, multi_signature: AggregateSignature) -> Self {
        Self {
            signed_node_digest_info,
            multi_signature,
        }
    }

    pub fn digest(&self) -> &HashValue {
        &self.signed_node_digest_info.digest
    }

    pub fn verify(&self, validator: &ValidatorVerifier) -> anyhow::Result<()> {
        validator
            .verify_multi_signatures(&self.signed_node_digest_info, &self.multi_signature)
            .context("Failed to verify ProofOfStore")
    }
}

#[allow(dead_code)]
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Node {
    epoch: u64,
    round: u64,
    source: PeerId,
    consensus_payload: Payload,
    parents: Vec<HashValue>,
    digest: HashValue,
}

impl Node {
    pub fn digest(&self) -> HashValue {
        self.digest
    }

    pub fn epoch(&self) -> u64 {
        self.epoch
    }

    pub fn round(&self) -> u64 {
        self.round
    }

    pub fn source(&self) -> PeerId {
        self.source
    }
}

#[allow(dead_code)]
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CertifiedNode {
    header: Node,
    certificate: NodeCertificate,
}

impl CertifiedNode {
    pub fn new(header: Node, certificate: NodeCertificate) -> Self {
        Self {
            header,
            certificate,
        }
    }

    pub fn node(&self) -> &Node {
        &self.header
    }
}

// TODO: check peer_id in msg.verify()
#[allow(dead_code)]
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CertifiedNodeAck {
    digest: HashValue,
    peer_id: PeerId,
}

impl CertifiedNodeAck {
    pub fn new(digest: HashValue, peer_id: PeerId) -> Self {
        Self {
            digest,
            peer_id,
        }
    }

    pub fn digest(&self) -> HashValue {
        self.digest
    }

    pub fn peer_id(&self) -> PeerId {
        self.peer_id
    }
}