// Copyright (c) Aptos
// SPDX-License-Identifier: Apache-2.0


pub(crate) struct Node {}

pub(crate) struct NodeCertificate {}

pub(crate) enum ReliableBroadcastCommand {
    BroadcastRequest(Node),
}