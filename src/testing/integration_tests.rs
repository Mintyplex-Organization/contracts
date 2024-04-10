#[cfg(test)]
mod tests {
    use crate::testing::helpers::MintyplexContract;

    use crate::msg::InstantiateMsg;
    use cosmwasm_std::{to_json_binary, Addr, Binary, Coin, Empty, Response, StdResult, Uint128};
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

    const USER: &str = "USER";
    const ADMIN: &str = "ADMIN";
    const NATIVE_DENOM: &str = "uxion";

    fn mock_app() -> App {
        AppBuilder::new().build(|router, _, storage| {
            router
                .bank
                .init_balance(
                    storage,
                    &Addr::unchecked(USER),
                    vec![Coin {
                        denom: NATIVE_DENOM.to_string(),
                        amount: Uint128::new(1),
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

    mod collection {
        use super::*;
        use crate::msg::{ExecuteMsg, QueryMsg};
        use crate::state::{CollectionParams, MintParams};

        #[test]
        fn create_collection() {
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
            };

            let msg = ExecuteMsg::CreateCollection(collection_params);
            let cosmos_msg = mintyplex_contract.call(msg).unwrap();
            app.execute(Addr::unchecked(USER), cosmos_msg).unwrap();

            // Query created collection
            let query_msg = QueryMsg::CreatorCollections {
                creator: Addr::unchecked(USER),
            };
            let creator_collections: Vec<Addr> = app
                .wrap()
                .query_wasm_smart(mintyplex_contract.addr(), &query_msg.clone())
                .unwrap();
            assert_eq!(creator_collections.len(), 1);

            let mint_params = MintParams {
                collection_address: creator_collections[0].clone(),
                code_id: cw721_code_id,
                owner: USER.to_string(),
                token_uri: "0".to_string(),
            };
            let msg = ExecuteMsg::MintNFT(mint_params);
            let cosmos_msg = mintyplex_contract.call(msg).unwrap();
            app.execute(Addr::unchecked(USER), cosmos_msg).unwrap();
        }
    }
}

// if info
// .funds
// .iter()
// .any(|coin| coin.denom == "uxion" && coin.amount.u128() == MINT_FEE)
// {
// return Err(ContractError::IncorrectFunds {});
// }
