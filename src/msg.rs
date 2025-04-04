use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr};

#[cw_serde]
pub struct InstantiateMsg {}

#[cw_serde]
pub enum ExecuteMsg {
    SetApprovalForAll { operator: Addr, approved: bool},
    SafeTransferFrom { from: Addr, to: Addr, id: u64, amount: u64 },
    SafeBatchTransferFrom { from: Addr, to: Addr, ids: Vec<u64>, amounts: Vec<u64> },
    Mint { to: Addr, id: u64, amount: u64 },
    MintBatch { to: Addr, ids: Vec<u64>, amounts: Vec<u64> },
    Burn { from: Addr, id: u64, amount: u64 },
    BurnBatch { from: Addr, ids: Vec<u64>, amounts: Vec<u64> }, 
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(BalanceOfResponse)]
    BalanceOf { account: Addr, id: u64 },

    #[returns(BalanceOfBatchResponse)]
    BalanceOfBatch { accounts: Vec<Addr>, ids: Vec<u64> },

    #[returns(IsApprovedForAllResponse)]
    IsApprovedForAll { account: Addr, operator: Addr },
}

#[cw_serde]
pub struct BalanceOfResponse {
    pub balanceOf: u64,
}

#[cw_serde]
pub struct BalanceOfBatchResponse {
    pub balanceOfBatch: Vec<u64>,
}

#[cw_serde]
pub struct IsApprovedForAllResponse {
    pub isApprovedForAll: bool,
}