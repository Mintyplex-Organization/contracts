use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, StdResult, Storage};
use cw_storage_plus::{Item, Map};
use serde::{Deserialize, Serialize};

#[cw_serde]
pub struct Config {
    pub owner: Addr,
    pub mint_percent: u128,
}

#[cw_serde]
#[derive(Default)]
pub struct CollectionInfo {
    pub name: String,
    pub symbol: String,
    pub mint_fee: u128,
    pub collection_address: Option<Addr>,
}

#[cw_serde]
pub struct CollectionParams {
    /// The collection code id
    pub code_id: u64,
    pub name: String,
    pub symbol: String,
    pub mint_fee: u128,
}

#[cw_serde]
pub struct MintParams {
    pub collection_creator: Addr,
    pub collection_name: String,
    pub collection_address: Addr,
    pub code_id: u64,
    pub owner: String,
    pub token_uri: String,
}

#[cw_serde]
pub struct UpdateMintFeeParams {
    pub collection_name: String,
    pub mint_fee: u128,
}

#[cw_serde]
pub struct WithdrawParams {
    pub withdraw_amount: u128,
    pub withdraw_address: Addr,
}

pub type CreatorAddress = Addr;
pub const CREATOR_COLLECTIONS: Map<(&CreatorAddress, &str), CollectionInfo> =
    Map::new("creator_collections");

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct PendingInstantiation {
    pub creator: Addr,
    pub collection_name: String,
}
pub const PENDING_INSTANTIATIONS: Map<u64, PendingInstantiation> =
    Map::new("pending_instantiations");

pub const CONFIG: Item<Config> = Item::new("config");

/// This keeps track of the token index for the token_ids
pub const TOKEN_INDEX: Item<u64> = Item::new("token_index");

pub fn increment_token_index(store: &mut dyn Storage) -> StdResult<u64> {
    let val = TOKEN_INDEX.may_load(store)?.unwrap_or_default() + 1;
    TOKEN_INDEX.save(store, &val)?;
    Ok(val)
}

pub const CW721_REPLY_ID: Item<u64> = Item::new("cw721_reply_id");

pub fn increment_reply_id(store: &mut dyn Storage) -> StdResult<u64> {
    let val = CW721_REPLY_ID.may_load(store)?.unwrap_or_default() + 1;
    CW721_REPLY_ID.save(store, &val)?;
    Ok(val)
}
