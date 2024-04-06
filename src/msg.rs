use crate::state;
use crate::state::Config;
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Addr;
use state::{CollectionParams, MintParams};

#[cw_serde]
pub struct InstantiateMsg {
    pub owner: Option<String>,
}

#[cw_serde]
pub enum ExecuteMsg {
    CreateCollection(CollectionParams),
    MintNFT(MintParams),
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(ConfigResponse)]
    QueryConfig {},
}

#[cw_serde]
pub struct ConfigResponse {
    pub config: Config,
    pub collection_address: Addr,
}
