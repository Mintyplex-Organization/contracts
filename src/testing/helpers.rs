#![allow(dead_code)]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{to_json_binary, Addr, Coin, CosmosMsg, StdResult, WasmMsg};

use crate::msg::ExecuteMsg;
use crate::state::MintParams;
use crate::testing::constants::{CREATOR, MOCK_COLLECTION_NAME, SHOPPER};

/// MintyplexContract is a wrapper around Addr that provides a lot of helpers
/// for working with this.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct MintyplexContract(pub Addr);

impl MintyplexContract {
    pub fn addr(&self) -> Addr {
        self.0.clone()
    }

    pub fn get_mock_mint_params(collection_address: Addr, code_id: u64) -> MintParams {
        MintParams {
            collection_creator: Addr::unchecked(CREATOR),
            collection_name: MOCK_COLLECTION_NAME.to_string(),
            collection_address,
            code_id,
            owner: SHOPPER.to_string(),
            token_uri: "0".to_string(),
        }
    }

    pub fn call<T: Into<ExecuteMsg>>(&self, msg: T) -> StdResult<CosmosMsg> {
        let msg = to_json_binary(&msg.into())?;
        Ok(WasmMsg::Execute {
            contract_addr: self.addr().into(),
            msg,
            funds: vec![],
        }
        .into())
    }

    pub fn call_with_funds<T: Into<ExecuteMsg>>(
        &self,
        msg: T,
        funds: Vec<Coin>,
    ) -> StdResult<CosmosMsg> {
        let msg = to_json_binary(&msg.into())?;
        Ok(WasmMsg::Execute {
            contract_addr: self.addr().into(),
            msg,
            funds,
        }
        .into())
    }
}
