use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, StdResult, Storage};
use cw_storage_plus::{Item, Map};
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
// ExecuteMsg::Mint{token_id, owner, token_uri}
#[cw_serde]
pub struct MintParams {
    pub code_id: u64,
    pub token_id: String,
    pub owner: String,
    pub token_uri: String,
}
pub const CONFIG: Item<Config> = Item::new("config");
pub const COLLECTION_ADDRESS: Item<Addr> = Item::new("collection_address");

pub const COLLECTIONS: Map<Addr, Addr> = Map::new("collections");

/// This keeps track of the token index for the token_ids
pub const TOKEN_INDEX: Item<u64> = Item::new("token_index");

pub fn increment_token_index(store: &mut dyn Storage) -> StdResult<u64> {
    let val = TOKEN_INDEX.may_load(store)?.unwrap_or_default() + 1;
    TOKEN_INDEX.save(store, &val)?;
    Ok(val)
}
