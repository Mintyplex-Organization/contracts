#![cfg(test)]
// use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
//
// use cosmwasm_std::{
//     from_json, to_json_binary, Addr, Coin, CosmosMsg, DepsMut, Empty, Response, StdError, WasmMsg,
// };

// #[cfg(test)]
mod tests {
    use crate::contract::{execute_create_collection, instantiate};
    use crate::msg;
    use crate::state::{CollectionParams, Config, CONFIG};
    use crate::ContractError;
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

    #[test]
    fn collection_creation() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = mock_info("creator", &[]);
        let _msg = InstantiateMsg { owner: None };

        let cw721_non_transferable_code_id = 465;

        let params: CollectionParams = CollectionParams {
            code_id: cw721_non_transferable_code_id,
            name: "book".to_string(),
            symbol: "book".to_string(),
        };

        let invalid_params: CollectionParams = CollectionParams {
            code_id: cw721_non_transferable_code_id,
            name: "".to_string(),
            symbol: "".to_string(),
        };

        let err =
            execute_create_collection(deps.as_mut(), env.clone(), info.clone(), invalid_params)
                .unwrap_err();
        assert_eq!(err, ContractError::InvalidInput {});

        let _res = execute_create_collection(deps.as_mut(), env.clone(), info.clone(), params);
    }
}
