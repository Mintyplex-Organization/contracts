#[cfg(test)]
mod tests {
    use crate::testing::helpers::MintyplexContract;

    use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
    use crate::state::{CollectionInfo, CollectionParams, MintParams};
    use cosmwasm_std::{
        coin, to_json_binary, Addr, Binary, Coin, Empty, Response, StdResult, Uint128,
    };
    use cw_multi_test::{App, AppBuilder, Contract, ContractWrapper, Executor};
    use serde::{Deserialize, Serialize};

    pub fn mintyplex_contract() -> Box<dyn Contract<Empty>> {
        let contract = ContractWrapper::new(
            crate::contract::execute,
            crate::contract::instantiate,
            crate::contract::query,
        )
        .with_reply(crate::contract::reply);
        Box::new(contract)
    }

    // Mock cw721 Contract
    #[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
    #[serde(rename_all = "snake_case")]
    pub enum Cw721MockExecuteMsg {
        /// Mint a new NFT, can only be called by the contract minter
        Mint {
            /// Unique ID of the NFT
            token_id: String,
            /// The owner of the newly minter NFT
            owner: String,
            /// Universal resource identifier for this NFT
            /// Should point to a JSON file that conforms to the ERC721
            /// Metadata JSON Schema
            token_uri: Option<String>,
        },
    }

    #[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
    #[serde(rename_all = "snake_case")]
    pub struct Cw721MockInstantiateMsg {}

    #[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
    #[serde(rename_all = "snake_case")]
    pub enum Cw721MockQueryMsg {}

    #[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
    #[serde(rename_all = "snake_case")]
    pub struct MockResponse {}

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

    const ADMIN: &str = "ADMIN";

    const CREATOR: &str = "CREATOR";

    const SHOPPER: &str = "SHOPPER";

    const NATIVE_DENOM: &str = "uxion";

    const MOCK_MINT_FEE: u128 = 1000000;

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

        let _cw_contract_addr = app
            .instantiate_contract(
                cw721_code_id,
                Addr::unchecked(ADMIN),
                &Cw721MockInstantiateMsg {},
                &[],
                "test",
                None,
            )
            .unwrap();

        let collection_params = CollectionParams {
            code_id: cw721_code_id,
            name: "product".to_string(),
            symbol: "PROD".to_string(),
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

        // Query created collection
        let query_msg = QueryMsg::CreatorCollections {
            creator: Addr::unchecked(CREATOR),
            collection_name: "product".to_string(),
        };
        let collection_info: CollectionInfo = app
            .wrap()
            .query_wasm_smart(
                mintyplex_contract_with_collection.addr(),
                &query_msg.clone(),
            )
            .unwrap();

        let mint_params = MintParams {
            collection_creator: Addr::unchecked(CREATOR),
            collection_name: "product".to_string(),
            collection_address: collection_info.collection_address.unwrap(),
            code_id: cw721_code_id,
            owner: SHOPPER.to_string(),
            token_uri: "0".to_string(),
        };

        let msg = ExecuteMsg::MintNFT(mint_params);

        let cosmos_msg = mintyplex_contract_with_collection
            .call_with_funds(msg, vec![coin(1000000, "uxion")])
            .unwrap();

        app.execute(Addr::unchecked(SHOPPER), cosmos_msg).unwrap();

        (app, mintyplex_contract_with_collection)
    }

    mod collection {
        use super::*;
        use crate::msg::{ExecuteMsg, QueryMsg};
        use crate::state::{
            CollectionInfo, CollectionParams, MintParams, UpdateMintFeeParams, WithdrawParams,
        };
        use cosmwasm_std::coin;

        #[test]
        fn test_create_collection() {
            let (mut app, mintyplex_contract) = proper_instantiate();

            let cw721_code_id = app.store_code(cw721_contract());

            let _cw_contract_addr = app
                .instantiate_contract(
                    cw721_code_id,
                    Addr::unchecked(ADMIN),
                    &Cw721MockInstantiateMsg {},
                    &[],
                    "test",
                    None,
                )
                .unwrap();

            let collection_params = CollectionParams {
                code_id: cw721_code_id,
                name: "product".to_string(),
                symbol: "PROD".to_string(),
                mint_fee: MOCK_MINT_FEE,
            };

            let msg = ExecuteMsg::CreateCollection(collection_params);
            let cosmos_msg = mintyplex_contract.call(msg).unwrap();
            app.execute(Addr::unchecked(CREATOR), cosmos_msg).unwrap();

            // Query created collection
            let query_msg = QueryMsg::CreatorCollections {
                creator: Addr::unchecked(CREATOR),
                collection_name: "product".to_string(),
            };
            let collection_info: StdResult<CollectionInfo> = app
                .wrap()
                .query_wasm_smart(mintyplex_contract.addr(), &query_msg.clone());
            assert!(collection_info.is_ok());
        }

        #[test]
        fn test_mint_nft() {
            let (mut app, mintyplex_contract_with_collection) = app_with_collection();

            let cw721_code_id = app.store_code(cw721_contract());

            // Query created collection
            let query_msg = QueryMsg::CreatorCollections {
                creator: Addr::unchecked(CREATOR),
                collection_name: "product".to_string(),
            };
            let collection_info: CollectionInfo = app
                .wrap()
                .query_wasm_smart(
                    mintyplex_contract_with_collection.addr(),
                    &query_msg.clone(),
                )
                .unwrap();

            let mint_params = MintParams {
                collection_creator: Addr::unchecked(CREATOR),
                collection_name: "product".to_string(),
                collection_address: collection_info.collection_address.unwrap(),
                code_id: cw721_code_id,
                owner: SHOPPER.to_string(),
                token_uri: "0".to_string(),
            };
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

            // when shopper passes the right amount
            let cosmos_msg = mintyplex_contract_with_collection
                .call_with_funds(msg, vec![coin(1000000, "uxion")])
                .unwrap();
            let res = app.execute(Addr::unchecked(SHOPPER), cosmos_msg).is_ok();
            assert!(res);
        }

        #[test]
        fn test_update_mint_fee() {
            let (mut app, mintyplex_contract_with_collection) = app_with_collection();

            // Query created collection
            let query_msg = QueryMsg::CreatorCollections {
                creator: Addr::unchecked(CREATOR),
                collection_name: "product".to_string(),
            };
            let collection_info: CollectionInfo = app
                .wrap()
                .query_wasm_smart(
                    mintyplex_contract_with_collection.addr(),
                    &query_msg.clone(),
                )
                .unwrap();

            assert_eq!(collection_info.mint_fee, MOCK_MINT_FEE);

            let new_mint_fee: u128 = 10;

            let update_mint_fee_params = UpdateMintFeeParams {
                collection_name: "product".to_string(),
                mint_fee: new_mint_fee,
            };

            let msg = ExecuteMsg::UpdateMintFee(update_mint_fee_params);

            let cosmos_msg = mintyplex_contract_with_collection.call(msg).unwrap();
            let res = app.execute(Addr::unchecked(CREATOR), cosmos_msg).unwrap();

            // Query created collection
            let query_msg = QueryMsg::CreatorCollections {
                creator: Addr::unchecked(CREATOR),
                collection_name: "product".to_string(),
            };
            let collection_info: CollectionInfo = app
                .wrap()
                .query_wasm_smart(
                    mintyplex_contract_with_collection.addr(),
                    &query_msg.clone(),
                )
                .unwrap();

            assert_eq!(collection_info.mint_fee, new_mint_fee);
        }

        // testing this has been difficult because this is private:https://docs.rs/cw-multi-test/0.20.0/src/cw_multi_test/bank.rs.html#64 :/
        // #[test]
        // fn test_withdraw() {
        //     let (mut app, mintyplex_contract_with_mint) = app_with_mint();
        //
        //     let storage = app.storage();
        //
        //     let withdraw_params = WithdrawParams {
        //         withdraw_amount: 20000,
        //         withdraw_address: Addr::unchecked(ADMIN),
        //     };
        //
        //     let msg = ExecuteMsg::Withdraw(withdraw_params);
        //
        //     let cosmos_msg = mintyplex_contract_with_mint.call(msg).unwrap();
        //     let res = app.execute(Addr::unchecked(ADMIN), cosmos_msg);
        //     // assert!(res);
        //     dbg!(res);
        //     assert!(false);
        // }
    }
}
