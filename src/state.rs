use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, StdResult, Storage};
use cw_storage_plus::{Item, Map};
use serde::{Deserialize, Serialize};

#[cw_serde]
pub struct Config {
    pub owner: Addr,
}

#[cw_serde]
pub struct CollectionParams {
    /// The collection code id
    pub code_id: u64,
    pub name: String,
    pub symbol: String,
}
#[cw_serde]
pub struct MintParams {
    pub collection_address: Addr,
    pub code_id: u64,
    pub owner: String,
    pub token_uri: String,
}
pub const CREATOR_COLLECTIONS: Map<&Addr, Vec<Addr>> = Map::new("creator_collections");

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct PendingInstantiation {
    pub creator: Addr,
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
