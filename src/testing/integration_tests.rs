#[cfg(test)]
mod tests {
    use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
    use crate::state::{
        CollectionInfo, CollectionParams, Config, UpdateMintFeeParams, WithdrawParams,
    };
    use crate::testing::constants::{
        ADMIN, ADMIN2, CREATOR, MOCK_COLLECTION_NAME, MOCK_COLLECTION_SYMBOL, MOCK_MINT_FEE,
        MOCK_MINT_PERCENT, NATIVE_DENOM, SHOPPER, UNAUTHORIZED,
    };
    use crate::testing::helpers::MintyplexContract;
    use crate::testing::types::{
        Cw721MockExecuteMsg, Cw721MockInstantiateMsg, Cw721MockQueryMsg, MockResponse,
    };
    use cosmwasm_std::{
        coin, to_json_binary, Addr, Binary, Coin, Empty, Response, StdResult, Uint128,
    };
    use cw_multi_test::{App, AppBuilder, Contract, ContractWrapper, Executor};

    pub fn mintyplex_contract() -> Box<dyn Contract<Empty>> {
        let contract = ContractWrapper::new(
            crate::contract::execute,
            crate::contract::instantiate,
            crate::contract::query,
        )
        .with_reply(crate::contract::reply);
        Box::new(contract)
    }

    pub fn cw721_contract() -> Box<dyn Contract<Empty>> {
        let contract = ContractWrapper::new(
            |_deps, _, _info, _msg: Cw721MockExecuteMsg| -> StdResult<Response> {
                Ok(Response::default())
            },
            |_, _, _, _: Cw721MockInstantiateMsg| -> StdResult<Response> {
                Ok(Response::default())
            },
            |_, _, _msg: Cw721MockQueryMsg| -> StdResult<Binary> {
                Ok(to_json_binary(&MockResponse {})?)
            },
        );
        Box::new(contract)
    }

    pub fn get_collection_info(
        app: &App,
        contract: &MintyplexContract,
    ) -> StdResult<CollectionInfo> {
        let query_msg = QueryMsg::CreatorCollections {
            creator: Addr::unchecked(CREATOR),
            collection_name: MOCK_COLLECTION_NAME.to_string(),
        };

        app.wrap().query_wasm_smart(contract.addr(), &query_msg)
    }

    fn mock_app() -> App {
        AppBuilder::new().build(|router, _, storage| {
            router
                .bank
                .init_balance(
                    storage,
                    &Addr::unchecked(SHOPPER),
                    vec![Coin {
                        denom: NATIVE_DENOM.to_string(),
                        amount: Uint128::new(100000000),
                    }],
                )
                .unwrap();
        })
    }

    fn proper_instantiate() -> (App, MintyplexContract) {
        let mut app = mock_app();

        let mintyplex_code_id = app.store_code(mintyplex_contract());

        let msg = InstantiateMsg {
            owner: Some(ADMIN.to_string()),
            mint_percent: MOCK_MINT_PERCENT,
        };
        let mintyplex_contract_addr = app
            .instantiate_contract(
                mintyplex_code_id,
                Addr::unchecked(ADMIN),
                &msg,
                &[],
                "test",
                None,
            )
            .unwrap();

        let mintyplex_contract = MintyplexContract(mintyplex_contract_addr);

        (app, mintyplex_contract)
    }

    fn app_with_collection() -> (App, MintyplexContract) {
        let (mut app, mintyplex_contract) = proper_instantiate();

        let cw721_code_id = app.store_code(cw721_contract());

        let collection_params = CollectionParams {
            code_id: cw721_code_id,
            name: MOCK_COLLECTION_NAME.to_string(),
            symbol: MOCK_COLLECTION_SYMBOL.to_string(),
            mint_fee: MOCK_MINT_FEE,
        };

        let msg = ExecuteMsg::CreateCollection(collection_params);
        let cosmos_msg = mintyplex_contract.call(msg).unwrap();
        app.execute(Addr::unchecked(CREATOR), cosmos_msg).unwrap();

        (app, mintyplex_contract)
    }

    fn app_with_mint() -> (App, MintyplexContract) {
        let (mut app, mintyplex_contract_with_collection) = app_with_collection();

        let cw721_code_id = app.store_code(cw721_contract());

        let collection_info =
            get_collection_info(&app, &mintyplex_contract_with_collection).unwrap();

        let mint_params = MintyplexContract::get_mock_mint_params(
            collection_info.collection_address.unwrap(),
            cw721_code_id,
        );

        let msg = ExecuteMsg::MintNFT(mint_params);

        let cosmos_msg = mintyplex_contract_with_collection
            .call_with_funds(msg, vec![coin(1000000, "uxion")])
            .unwrap();

        app.execute(Addr::unchecked(SHOPPER), cosmos_msg).unwrap();

        (app, mintyplex_contract_with_collection)
    }

    mod collection {
        use super::*;

        #[test]
        fn test_create_collection() {
            let (mut app, mintyplex_contract) = proper_instantiate();

            let cw721_code_id = app.store_code(cw721_contract());

            let collection_params = CollectionParams {
                code_id: cw721_code_id,
                name: MOCK_COLLECTION_NAME.to_string(),
                symbol: MOCK_COLLECTION_SYMBOL.to_string(),
                mint_fee: MOCK_MINT_FEE,
            };

            let msg = ExecuteMsg::CreateCollection(collection_params);
            let cosmos_msg = mintyplex_contract.call(msg).unwrap();
            app.execute(Addr::unchecked(CREATOR), cosmos_msg).unwrap();

            let collection_info: StdResult<CollectionInfo> =
                get_collection_info(&app, &mintyplex_contract);
            assert!(collection_info.is_ok());
        }

        #[test]
        fn test_mint_nft() {
            let (mut app, mintyplex_contract_with_collection) = app_with_collection();

            let cw721_code_id = app.store_code(cw721_contract());

            let collection_info =
                get_collection_info(&app, &mintyplex_contract_with_collection).unwrap();

            let mint_params = MintyplexContract::get_mock_mint_params(
                collection_info.collection_address.unwrap(),
                cw721_code_id,
            );
            let msg = ExecuteMsg::MintNFT(mint_params);

            // test if shopper passes the wrong amount
            let cosmos_msg_with_wrong_amount = mintyplex_contract_with_collection
                .call_with_funds(msg.clone(), vec![coin(100000, "uxion")])
                .unwrap();

            let err_res = app
                .execute(Addr::unchecked(SHOPPER), cosmos_msg_with_wrong_amount)
                .is_err();

            assert!(err_res);

            // test when shopper passes the right amount but wrong denom
            let cosmos_msg_with_wrong_denom = mintyplex_contract_with_collection
                .call_with_funds(msg.clone(), vec![coin(1000000, "wrong_denom")])
                .unwrap();

            let err_res = app
                .execute(Addr::unchecked(SHOPPER), cosmos_msg_with_wrong_denom)
                .is_err();

            assert!(err_res);

            // when shopper passes the right amount and right denom
            let cosmos_msg = mintyplex_contract_with_collection
                .call_with_funds(msg, vec![coin(MOCK_MINT_FEE, "uxion")])
                .unwrap();
            let res = app.execute(Addr::unchecked(SHOPPER), cosmos_msg).is_ok();
            assert!(res);
        }
        #[test]
        fn test_withdraw() {
            let (mut app, mintyplex_contract_with_mint) = app_with_mint();

            assert!(app
                .wrap()
                .query_balance(Addr::unchecked(ADMIN), NATIVE_DENOM)
                .unwrap()
                .amount
                .is_zero());

            let contract_balance = app
                .wrap()
                .query_balance(mintyplex_contract_with_mint.addr(), NATIVE_DENOM)
                .unwrap()
                .amount;

            let withdraw_params = WithdrawParams {
                withdraw_amount: u128::from(contract_balance),
                withdraw_address: Addr::unchecked(ADMIN),
            };

            let msg = ExecuteMsg::Withdraw(withdraw_params);

            let cosmos_msg = mintyplex_contract_with_mint.call(msg).unwrap();

            // should fail with unauthorized address
            let res_err = app
                .execute(Addr::unchecked(UNAUTHORIZED), cosmos_msg.clone())
                .is_err();
            assert!(res_err);

            let _ = app.execute(Addr::unchecked(ADMIN), cosmos_msg);

            assert_eq!(
                app.wrap()
                    .query_balance(Addr::unchecked(ADMIN), "uxion")
                    .unwrap()
                    .amount,
                contract_balance
            );
        }

        #[test]
        fn test_update_config() {
            let (mut app, mintyplex_contract) = proper_instantiate();

            let new_config = Config {
                owner: Addr::unchecked(ADMIN2),
                mint_percent: 0,
            };

            let msg = ExecuteMsg::UpdateConfig(new_config.clone());

            let cosmos_msg = mintyplex_contract.call(msg).unwrap();

            // should fail with unauthorized address
            let res_err = app
                .execute(Addr::unchecked(UNAUTHORIZED), cosmos_msg.clone())
                .is_err();
            assert!(res_err);

            let _ = app
                .execute(Addr::unchecked(ADMIN), cosmos_msg.clone())
                .unwrap();

            let query_msg = QueryMsg::Config {};
            let current_config: Config = app
                .wrap()
                .query_wasm_smart(mintyplex_contract.addr(), &query_msg)
                .unwrap();

            assert_eq!(new_config, current_config);
        }

        #[test]
        fn test_update_mint_fee() {
            let (mut app, mintyplex_contract_with_collection) = app_with_collection();

            let collection_info =
                get_collection_info(&app, &mintyplex_contract_with_collection).unwrap();

            assert_eq!(collection_info.mint_fee, MOCK_MINT_FEE);

            let new_mint_fee: u128 = 10;

            let update_mint_fee_params = UpdateMintFeeParams {
                collection_name: MOCK_COLLECTION_NAME.to_string(),
                mint_fee: new_mint_fee,
            };

            let msg = ExecuteMsg::UpdateMintFee(update_mint_fee_params);

            let cosmos_msg = mintyplex_contract_with_collection.call(msg).unwrap();

            // should fail with unauthorized address
            let res_err = app
                .execute(Addr::unchecked(UNAUTHORIZED), cosmos_msg.clone())
                .is_err();
            assert!(res_err);

            let _ = app.execute(Addr::unchecked(CREATOR), cosmos_msg).unwrap();

            let collection_info =
                get_collection_info(&app, &mintyplex_contract_with_collection).unwrap();

            assert_eq!(collection_info.mint_fee, new_mint_fee);
        }
    }
}
