use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Invalid Address")]
    InvalidAddress {},

    #[error("Ids and Amounts are not equal is length")]
    InvalidIdAmountLength {},

    #[error("Insufficient Balance")]
    InsufficientBalance {},

    #[error("Owner and Operator are same")]
    SelfApprovedError {},
}