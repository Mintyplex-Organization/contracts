#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_json_binary, Addr, Binary, CosmosMsg, Deps, DepsMut, Empty, Env, MessageInfo, Reply,
    Response, StdResult, SubMsg, WasmMsg,
};
use cw2::set_contract_version;
use cw721_base::Extension;
use cw721_non_transferable::{
    ExecuteMsg as Cw721ExecuteMsg, InstantiateMsg as cw721NonTransferableInstantiateMsg,
};
use cw_utils::parse_reply_instantiate_data;

use crate::error::ContractError;
use crate::msg::{ConfigResponse, ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{
    increment_token_index, CollectionParams, Config, MintParams, COLLECTION_ADDRESS, CONFIG,
};
use url::Url;

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:contracts";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

const INSTANTIATE_CW721_REPLY_ID: u64 = 1;

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

pub fn execute_create_collection(
    _deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    params: CollectionParams,
) -> Result<Response, ContractError> {
    let wasm_msg = WasmMsg::Instantiate {
        admin: None,
        code_id: params.code_id,
        msg: to_json_binary(&cw721NonTransferableInstantiateMsg {
            admin: None,
            name: params.name.clone(),
            symbol: params.symbol,
            minter: info.sender.to_string(),
        })?,

        funds: info.funds,
        label: format!("CW721-{}-{}", params.code_id, params.name.trim()),
    };

    let submsg = SubMsg::reply_on_success(wasm_msg, INSTANTIATE_CW721_REPLY_ID);

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
    let collection_address = COLLECTION_ADDRESS.load(deps.storage)?;
    // check if caller is owner of collection

    Url::parse(&params.token_uri).map_err(|_| ContractError::InvalidTokenURI {})?;

    let mut res = Response::new();

    // Create mint msgs
    let mint_msg = Cw721ExecuteMsg::<Extension, Empty>::Mint {
        token_id: increment_token_index(deps.storage)?.to_string(),
        owner: info.sender.to_string(),
        token_uri: Some(params.token_uri.clone()),
        extension: None,
    };
    let msg = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: collection_address.to_string(),
        msg: to_json_binary(&mint_msg)?,
        funds: vec![],
    });
    res = res.add_message(msg);

    Ok(res.add_attribute("action", "mint-nft"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::QueryConfig {} => to_json_binary(&query_config(deps)?),
    }
}

fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let config = CONFIG.load(deps.storage)?;
    let collection_address = COLLECTION_ADDRESS.load(deps.storage)?;

    Ok(ConfigResponse {
        collection_address,
        config,
    })
}

// Reply callback triggered from cw721 contract instantiation in instantiate()
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, _env: Env, msg: Reply) -> Result<Response, ContractError> {
    if msg.id != INSTANTIATE_CW721_REPLY_ID {
        return Err(ContractError::InvalidReplyID {});
    }

    let reply = parse_reply_instantiate_data(msg);
    match reply {
        Ok(res) => {
            let collection_address = res.contract_address;
            COLLECTION_ADDRESS.save(deps.storage, &Addr::unchecked(collection_address.clone()))?;
            Ok(Response::default()
                .add_attribute("action", "instantiate_cw721_reply")
                .add_attribute("cw721_address", collection_address))
        }
        Err(_) => Err(ContractError::InstantiateCw721Error {}),
    }
}
#[cfg(test)]
mod tests {
    use crate::contract::instantiate;
    use crate::msg;
    use crate::state::{CollectionParams, Config, CONFIG};
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::Addr;
    use msg::InstantiateMsg;

    #[test]
    fn proper_instantiation() {
        let mut deps = mock_dependencies();
        let msg = InstantiateMsg { owner: None };
        let env = mock_env();
        let info = mock_info("creator", &[]);

        let res = instantiate(deps.as_mut(), env.clone(), info.clone(), msg)
            .expect("instantiation failed");
        assert_eq!(0, res.messages.len());

        let state = CONFIG.load(&deps.storage).unwrap();
        assert_eq!(
            state,
            Config {
                owner: Addr::unchecked("creator".to_string())
            }
        )
    }

    // #[test]
    // fn collection_creation() {
    //     let mut deps = mock_dependencies();
    //     let env = mock_env();
    //     let info = mock_info("creator", &[]);
    // }
}
