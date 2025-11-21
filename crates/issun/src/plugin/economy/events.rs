//! Events for economy plugin

use super::types::{Currency, CurrencyId, ResourceId};
use crate::event::Event;
use serde::{Deserialize, Serialize};

// ============================================================================
// Command Events (Request)
// ============================================================================

/// Request to exchange currency
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurrencyExchangeRequested {
    pub from_currency: CurrencyId,
    pub to_currency: CurrencyId,
    pub from_amount: Currency,
}

impl Event for CurrencyExchangeRequested {}

/// Request to convert resource to currency
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceConversionRequested {
    pub resource_id: ResourceId,
    pub currency_id: CurrencyId,
    pub resource_amount: i64,
}

impl Event for ResourceConversionRequested {}

/// Request to add resource to inventory
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceAddRequested {
    pub resource_id: ResourceId,
    pub amount: i64,
}

impl Event for ResourceAddRequested {}

/// Request to consume resource from inventory
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceConsumeRequested {
    pub resource_id: ResourceId,
    pub amount: i64,
}

impl Event for ResourceConsumeRequested {}

// ============================================================================
// State Events (Notification)
// ============================================================================

/// Currency was exchanged successfully
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurrencyExchanged {
    pub from_currency: CurrencyId,
    pub to_currency: CurrencyId,
    pub from_amount: Currency,
    pub to_amount: Currency,
}

impl Event for CurrencyExchanged {}

/// Currency exchange failed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurrencyExchangeFailed {
    pub from_currency: CurrencyId,
    pub to_currency: CurrencyId,
    pub from_amount: Currency,
    pub reason: String,
}

impl Event for CurrencyExchangeFailed {}

/// Resource was converted to currency successfully
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceConverted {
    pub resource_id: ResourceId,
    pub currency_id: CurrencyId,
    pub resource_amount: i64,
    pub currency_amount: Currency,
}

impl Event for ResourceConverted {}

/// Resource conversion failed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceConversionFailed {
    pub resource_id: ResourceId,
    pub currency_id: CurrencyId,
    pub resource_amount: i64,
    pub reason: String,
}

impl Event for ResourceConversionFailed {}

/// Resource was added to inventory
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceAdded {
    pub resource_id: ResourceId,
    pub amount: i64,
    pub new_total: i64,
}

impl Event for ResourceAdded {}

/// Resource was consumed from inventory
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceConsumed {
    pub resource_id: ResourceId,
    pub amount: i64,
    pub remaining: i64,
}

impl Event for ResourceConsumed {}

/// Resource consumption failed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceConsumeFailed {
    pub resource_id: ResourceId,
    pub amount: i64,
    pub reason: String,
}

impl Event for ResourceConsumeFailed {}

/// Flow resource generated currency (automatic per-turn generation)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlowResourceGenerated {
    pub resource_id: ResourceId,
    pub currency_id: CurrencyId,
    pub resource_capacity: i64,
    pub currency_generated: Currency,
}

impl Event for FlowResourceGenerated {}

/// Currency deposited to wallet
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurrencyDeposited {
    pub currency_id: CurrencyId,
    pub amount: Currency,
    pub new_balance: Currency,
}

impl Event for CurrencyDeposited {}

/// Currency withdrawn from wallet
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurrencyWithdrawn {
    pub currency_id: CurrencyId,
    pub amount: Currency,
    pub new_balance: Currency,
}

impl Event for CurrencyWithdrawn {}
