// Copyright © Aptos Foundation
// SPDX-License-Identifier: Apache-2.0

use tonic::{Code, Status};

#[derive(Debug)]
pub struct ApiError {
    status: Status,
}

impl From<ApiError> for Status {
    fn from(err: ApiError) -> Self {
        err.status
    }
}

impl From<Status> for ApiError {
    fn from(status: Status) -> Self {
        Self { status }
    }
}

// I wish it was possible to build a Status with both a Code and an error. It seems like you
// can only do one or the other:
// 1. Pass a code and a string message.
// 2. Pass an error (so you get a `source` in the Status) but you can't set the code.
impl ApiError {
    pub fn new(code: Code, message: impl Into<String>) -> Self {
        Self {
            status: Status::new(code, message),
        }
    }
}
