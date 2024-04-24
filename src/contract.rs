use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{
    increment_reply_id, increment_token_index, CollectionInfo, CollectionParams, Config,
    MintParams, PendingInstantiation, UpdateMintFeeParams, WithdrawParams, CONFIG,
    CREATOR_COLLECTIONS, CW721_REPLY_ID, PENDING_INSTANTIATIONS, TOKEN_INDEX,
};
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
        mint_percent: 2,
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
        ExecuteMsg::UpdateMintFee(params) => execute_update_mint_fee(deps, env, info, params),
        ExecuteMsg::Withdraw(params) => execute_withdraw(deps, env, info, params),
    }
}

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
        collection_name: params.name.clone(),
    };

    PENDING_INSTANTIATIONS.save(deps.storage, reply_id, &pending)?;

    let collection_info = CollectionInfo {
        name: params.name.clone(),
        symbol: params.symbol.clone(),
        mint_fee: params.mint_fee,
        collection_address: None,
    };

    CREATOR_COLLECTIONS.save(
        deps.storage,
        (&info.sender, &params.name.clone()),
        &collection_info,
    )?;

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
    _env: Env,
    info: MessageInfo,
    params: MintParams,
) -> Result<Response, ContractError> {
    let mint_fee = CREATOR_COLLECTIONS
        .load(
            deps.storage,
            (&params.collection_creator, &params.collection_name),
        )?
        .mint_fee;

    if info
        .funds
        .iter()
        .any(|coin| coin.denom != "uxion" && coin.amount.u128() != mint_fee)
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

    let mint_percent = CONFIG.load(deps.storage)?.mint_percent;

    let mintyplex_amount = (mint_fee * mint_percent) / 100;
    let creator_amount = mint_fee - mintyplex_amount;

    let mintyplex_bank_msg = BankMsg::Send {
        to_address: params.collection_creator.to_string(),
        amount: vec![Coin {
            denom: "uxion".to_string(),
            amount: Uint128::from(mintyplex_amount),
        }],
    };

    let creator_bank_msg = BankMsg::Send {
        to_address: params.collection_creator.to_string(),
        amount: vec![Coin {
            denom: "uxion".to_string(),
            amount: Uint128::from(creator_amount),
        }],
    };

    Ok(Response::new()
        .add_message(msg)
        .add_message(mintyplex_bank_msg)
        .add_message(creator_bank_msg)
        .add_attribute("action", "mint nft"))
}

pub fn execute_update_mint_fee(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    params: UpdateMintFeeParams,
) -> Result<Response, ContractError> {
    let mut collection_info =
        CREATOR_COLLECTIONS.load(deps.storage, (&info.sender, &params.collection_name))?;

    collection_info.mint_fee = params.mint_fee;

    CREATOR_COLLECTIONS.save(
        deps.storage,
        (&info.sender, &params.collection_name),
        &collection_info,
    )?;

    Ok(Response::new().add_attribute("action", "update mint fee"))
}

pub fn execute_withdraw(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    params: WithdrawParams,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;

    if info.sender != config.owner {
        return Err(ContractError::Unauthorized {});
    }

    let bank_msg = BankMsg::Send {
        to_address: params.withdraw_address.to_string(),
        amount: vec![Coin {
            denom: "uxion".to_string(),
            amount: Uint128::from(params.withdraw_amount),
        }],
    };

    Ok(Response::new()
        .add_message(bank_msg)
        .add_attribute("action", "withdraw"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_json_binary(&query_config(deps)?),
        QueryMsg::TokenIndex {} => to_json_binary(&query_token_index(deps)?),
        QueryMsg::CreatorCollections {
            creator,
            collection_name,
        } => to_json_binary(&query_creator_collections(deps, creator, collection_name)?),
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

fn query_creator_collections(
    deps: Deps,
    creator: Addr,
    collection_name: String,
) -> StdResult<CollectionInfo> {
    let collection_info = CREATOR_COLLECTIONS
        .may_load(deps.storage, (&creator, &collection_name))?
        .unwrap_or_default();
    Ok(collection_info)
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

    match parse_reply_instantiate_data(msg.clone()) {
        Ok(res) => {
            let collection_address = res.contract_address;

            let mut collections = CREATOR_COLLECTIONS
                .may_load(deps.storage, (&pending.creator, &pending.collection_name))?
                .unwrap();

            collections.collection_address =
                Some(deps.api.addr_validate(&collection_address).unwrap());

            CREATOR_COLLECTIONS.save(
                deps.storage,
                (&pending.creator, &pending.collection_name),
                &collections,
            )?;

            PENDING_INSTANTIATIONS.remove(deps.storage, msg.id);

            Ok(Response::default()
                .add_attribute("action", "instantiate_cw721_reply")
                .add_attribute("cw721_address", collection_address))
        }
        Err(_) => Err(ContractError::InstantiateCw721Error {}),
    }
}
