use crate::state::{self, CollectionInfo};
use crate::state::{Config, UpdateMintFeeParams, WithdrawParams};
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Addr;
use state::{CollectionParams, MintParams};

#[cw_serde]
pub struct InstantiateMsg {
    pub owner: Option<String>,
    pub mint_percent: u128,
}

#[cw_serde]
pub enum ExecuteMsg {
    CreateCollection(CollectionParams),
    MintNFT(MintParams),
    Withdraw(WithdrawParams),
    UpdateConfig(Config),
    UpdateMintFee(UpdateMintFeeParams),
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    // Query for config
    #[returns(ConfigResponse)]
    Config {},

    // Query for the current token index
    #[returns(u64)]
    TokenIndex {},

    // Query for collections created by a specific creator
    #[returns(CollectionInfoResponse)]
    CreatorCollections {
        creator: Addr,
        collection_name: String,
    },
}

// We define a custom struct for each query response
#[cw_serde]
pub struct CollectionInfoResponse {
    pub name: String,
    pub symbol: String,
    pub mint_fee: u128,
    pub collection_address: Option<Addr>,
}

impl From<CollectionInfo> for CollectionInfoResponse {
    fn from(collection_info: CollectionInfo) -> CollectionInfoResponse {
        CollectionInfoResponse {
            name: collection_info.name,
            symbol: collection_info.symbol,
            mint_fee: collection_info.mint_fee,
            collection_address: collection_info.collection_address,
        }
    }
}

#[cw_serde]
pub struct ConfigResponse {
    pub owner: Addr,
    pub mint_percent: u128,
}

impl From<Config> for ConfigResponse {
    fn from(config: Config) -> ConfigResponse {
        ConfigResponse {
            owner: config.owner,
            mint_percent: config.mint_percent,
        }
    }
}
