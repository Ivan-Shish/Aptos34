// Copyright (c) Aptos
// SPDX-License-Identifier: Apache-2.0


use move_core_types::effects::Op;
use aptos_consensus_types::common::{Payload, Round};
use aptos_crypto::{bls12381, HashValue};
use aptos_types::aggregate_signature::AggregateSignature;
use aptos_types::PeerId;
use crate::common::Payload;


#[derive(Deserialize, Serialize, Clone, Debug, PartialEq, Eq)]
pub struct NodeCertificate {
    digest: HashValue,
    multi_signature: AggregateSignature,
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