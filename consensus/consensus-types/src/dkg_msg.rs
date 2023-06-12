// Copyright © Aptos Foundation
// SPDX-License-Identifier: Apache-2.0

use std::{thread, time::Duration};

use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DKGMsg(pub Vec<u8>);

pub enum RealDKGMsg {
    Transcript(Transcript),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Transcript {
    // dkg todo: use real transcript
    bytes: Vec<u8>,
}

impl Transcript {
    pub fn new(transcript_size: usize) -> Self {
        Transcript { bytes: vec![u8::MAX; transcript_size] }
    }

    pub fn verify(&self, transcript_verify_time_ms: u64) -> anyhow::Result<()> {
        // dkg todo: verify the transcript
        thread::sleep(Duration::from_millis(transcript_verify_time_ms));
        Ok(())
    }
}
