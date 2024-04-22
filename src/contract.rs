#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_json_binary, Addr, BankMsg, Binary, Coin, CosmosMsg, Deps, DepsMut, Empty, Env, MessageInfo,
    Reply, Response, StdResult, SubMsg, Uint128, WasmMsg,
};
use cw2::set_contract_version;
use cw721_base::Extension;
use cw721_non_transferable::{
    ExecuteMsg as Cw721ExecuteMsg, InstantiateMsg as cw721NonTransferableInstantiateMsg,
};
use cw_utils::parse_reply_instantiate_data;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{
    increment_reply_id, increment_token_index, CollectionParams, Config, MintParams,
    PendingInstantiation, CONFIG, CREATOR_COLLECTIONS, CW721_REPLY_ID, PENDING_INSTANTIATIONS,
    TOKEN_INDEX,
};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:mintyplex";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let owner = msg
        .owner
        .and_then(|addr_string| deps.api.addr_validate(addr_string.as_str()).ok())
        .unwrap_or(info.sender);

    let config = Config {
        owner: owner.clone(),
    };
    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("owner", owner))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::CreateCollection(params) => execute_create_collection(deps, env, info, params),
        ExecuteMsg::MintNFT(params) => execute_mint_nft(deps, env, info, params),
    }
}

const MINT_FEE: u128 = 1000000; // Define the mint fee

pub fn execute_create_collection(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    params: CollectionParams,
) -> Result<Response, ContractError> {
    if params.name.is_empty() || params.symbol.is_empty() {
        return Err(ContractError::InvalidInput {});
    }

    let reply_id = increment_reply_id(deps.storage)?;

    let pending = PendingInstantiation {
        creator: info.sender.clone(),
    };

    PENDING_INSTANTIATIONS.save(deps.storage, reply_id, &pending)?;

    let wasm_msg = WasmMsg::Instantiate {
        admin: None,
        code_id: params.code_id,
        msg: to_json_binary(&cw721NonTransferableInstantiateMsg {
            admin: None,
            name: params.name.clone(),
            symbol: params.symbol.clone(),
            minter: env.contract.address.to_string(),
        })?,

        funds: info.funds,
        label: format!("CW721-{}-{}", params.code_id, params.name.trim()),
    };

    let submsg = SubMsg::reply_on_success(wasm_msg, reply_id);

    Ok(Response::new()
        .add_attribute("action", "create collection")
        .add_submessage(submsg))
}

pub fn execute_mint_nft(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    params: MintParams,
) -> Result<Response, ContractError> {
    if info
        .funds
        .iter()
        .any(|coin| coin.denom != "uxion" && coin.amount.u128() != MINT_FEE)
    {
        return Err(ContractError::IncorrectFunds {});
    }

    // Create mint msg
    let mint_msg = Cw721ExecuteMsg::<Extension, Empty>::Mint {
        token_id: increment_token_index(deps.storage)?.to_string(),
        owner: info.sender.to_string(),
        token_uri: Some(params.token_uri),
        extension: None,
    };
    let msg = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: params.collection_address.to_string(),
        msg: to_json_binary(&mint_msg)?,
        funds: vec![],
    });

    let bank_msg = BankMsg::Send {
        to_address: env.contract.address.to_string(),
        amount: vec![Coin {
            denom: "uxion".to_string(),
            amount: Uint128::from(MINT_FEE),
        }],
    };

    Ok(Response::new()
        .add_message(msg)
        .add_message(bank_msg)
        .add_attribute("action", "mint-nft"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_json_binary(&query_config(deps)?),
        QueryMsg::TokenIndex {} => to_json_binary(&query_token_index(deps)?),
        QueryMsg::CreatorCollections { creator } => {
            to_json_binary(&query_creator_collections(deps, creator)?)
        }
    }
}

fn query_config(deps: Deps) -> StdResult<Config> {
    let config = CONFIG.load(deps.storage)?;
    Ok(config)
}

fn query_token_index(deps: Deps) -> StdResult<u64> {
    let index = TOKEN_INDEX.load(deps.storage)?;
    Ok(index)
}

fn query_creator_collections(deps: Deps, creator: Addr) -> StdResult<Vec<Addr>> {
    let collections = CREATOR_COLLECTIONS
        .may_load(deps.storage, &creator)?
        .unwrap_or_default();
    Ok(collections)
}

// Reply callback triggered from cw721 contract instantiation in instantiate()
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, _env: Env, msg: Reply) -> Result<Response, ContractError> {
    if msg.id != CW721_REPLY_ID.load(deps.storage)? {
        return Err(ContractError::InvalidReplyID {});
    }

    let pending = PENDING_INSTANTIATIONS
        .may_load(deps.storage, msg.id)?
        .ok_or(ContractError::InvalidReplyID {})?;
    let creator = pending.creator;

    match parse_reply_instantiate_data(msg.clone()) {
        Ok(res) => {
            let collection_address = res.contract_address;

            let mut collections = CREATOR_COLLECTIONS
                .may_load(deps.storage, &creator)?
                .unwrap_or_else(Vec::new);

            collections.push(deps.api.addr_validate(&collection_address).unwrap());

            CREATOR_COLLECTIONS.save(deps.storage, &creator, &collections)?;

            PENDING_INSTANTIATIONS.remove(deps.storage, msg.id);

            Ok(Response::default()
                .add_attribute("action", "instantiate_cw721_reply")
                .add_attribute("cw721_address", collection_address))
        }
        Err(_) => Err(ContractError::InstantiateCw721Error {}),
    }
}
