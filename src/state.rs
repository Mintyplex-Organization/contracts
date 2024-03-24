use cosmwasm_schema::cw_serde;
use cosmwasm_std::Addr;
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

pub const CONFIG: Item<Config> = Item::new("config");
pub const COLLECTION_ADDRESS: Item<Addr> = Item::new("collection_address");

