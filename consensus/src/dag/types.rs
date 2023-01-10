// Copyright (c) Aptos
// SPDX-License-Identifier: Apache-2.0

use std::collections::BTreeMap;
use aptos_consensus_types::node::{SignedNodeDigest, SignedNodeDigestError};
use aptos_crypto::{bls12381, HashValue};
use aptos_types::PeerId;
use aptos_types::validator_verifier::ValidatorVerifier;

struct IncrementalNodeCertificateState {
    digest: HashValue,
    aggregated_signature: BTreeMap<PeerId, bls12381::Signature>,
}

impl IncrementalNodeCertificateState {
    fn new(digest: HashValue) -> Self {
        Self {
            digest,
            aggregated_signature: BTreeMap::new(),
        }
    }

    fn missing_peers_signatures() {}

    //Signature we already verified
    fn add_signature(&mut self, signed_node_digest: SignedNodeDigest) -> Result<(), SignedNodeDigestError> {
        if signed_node_digest.digest() != &self.digest {
            return Err(SignedNodeDigestError::WrongDigest);
        }

        if self
            .aggregated_signature
            .contains_key(&signed_digest.peer_id())
        {
            return Err(SignedNodeDigestError::DuplicatedSignature);
        }

        self.aggregated_signature
            .insert(signed_node_digest.peer_id(), signed_node_digest.signature());
        Ok(())
    }

    fn ready(&self, validator_verifier: &ValidatorVerifier, my_peer_id: PeerId) -> bool {
        self.aggregated_signature.contains_key(&my_peer_id)
            && validator_verifier
            .check_voting_power(self.aggregated_signature.keys())
            .is_ok()
    }

    fn take(
        self,
        validator_verifier: &ValidatorVerifier,
    ) -> (ProofOfStore, BatchId, ProofReturnChannel) {
        let proof = match validator_verifier
            .aggregate_signatures(&PartialSignatures::new(self.aggregated_signature))
        {
            Ok(sig) => ProofOfStore::new(self.info, sig),
            Err(e) => unreachable!("Cannot aggregate signatures on digest err = {:?}", e),
        };
        (proof, self.batch_id, self.ret_tx)
    }

    fn send_timeout(self) {
        self.ret_tx
            .send(Err(ProofError::Timeout(self.batch_id)))
            .expect("Unable to send the timeout a proof of store");
    }
}