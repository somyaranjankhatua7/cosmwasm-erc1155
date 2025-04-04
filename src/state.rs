use cw_storage_plus::{ Map };
use cosmwasm_std::{ Addr };

pub const BALANCES: Map<(&str, &str), u64> = Map::new("balance");
pub const OPERATOR_APPROVAL: Map<(&Addr, &Addr), bool> = Map::new("allowance");