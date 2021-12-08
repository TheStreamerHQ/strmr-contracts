use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::serde::{Deserialize, Serialize};
use crate::*;

/// Represent the record what user is subscribed for
#[derive(BorshDeserialize, BorshSerialize, Deserialize, Serialize)]
#[serde(crate = "near_sdk::serde")]
pub struct UserSubscription {
    /// Generated uuid
    pub id: u8,
    /// Defines if the subscription is action
    pub enabled: bool,
    /// Endcoded webhook address
    pub endpoint: String,
    /// The event itself
    pub event: Event,
}

/// Represents the event which should be checked for a user
#[derive(BorshDeserialize, BorshSerialize, Deserialize, Serialize)]
#[serde(crate = "near_sdk::serde", tag = "kind", content = "triggers")]
pub enum Event {
    /// ExecutionOutcome for a Receipt with Transfer action(s)
    ReceiptTransferResult(Vec<TransferTrigger>),
    /// ExecutionOutcome for a Receipt with FunctionCall action(s)
    ReceiptFunctionCallResult(Vec<FunctionCallTrigger>),
}

#[derive(BorshDeserialize, BorshSerialize, Deserialize, Serialize)]
#[serde(crate = "near_sdk::serde", tag = "parameter", content = "value")]
pub enum TransferTrigger {
    SignerId(ValidAccountId),
    ReceiverId(ValidAccountId),
    AmountEqualOrGreater(U128),
    AmountEqualOrLower(U128),
}

#[derive(BorshDeserialize, BorshSerialize, Deserialize, Serialize)]
#[serde(crate = "near_sdk::serde", tag = "parameter", content = "value")]
pub enum FunctionCallTrigger {
    ReceiverId(ValidAccountId),
    FunctionNameExact(String),
    FunctionNameLike(String),
}
