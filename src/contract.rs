#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{Binary, to_binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult, Addr};
use cw2::set_contract_version;

use crate::state::{ BALANCES, OPERATOR_APPROVAL };
use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg, BalanceOfResponse, BalanceOfBatchResponse, IsApprovedForAllResponse};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:cw1155-base";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(_deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    Ok(Response::new().add_attribute("method", "instantiate"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match _msg {
        ExecuteMsg::SetApprovalForAll { operator, approved } => execute::set_approval_for_all(_deps, _info, operator, approved),
        ExecuteMsg::SafeTransferFrom { from, to, id, amount } => execute::safe_transfer_from(_deps, _info, from, to, id, amount),
        ExecuteMsg::SafeBatchTransferFrom { from, to, ids, amounts } => execute::safe_batch_transfer_from(_deps, _info, from, to, ids, amounts),
        ExecuteMsg::Mint { to, id, amount } => execute::mint(_deps, _info, to, id, amount),
        ExecuteMsg::MintBatch { to, ids, amounts } => execute::mint_batch(_deps, _info, to, ids, amounts),
        ExecuteMsg::Burn { from, id, amount } => execute::burn(_deps, _info, from, id, amount),
        ExecuteMsg::BurnBatch { from, ids, amounts } => execute::burn_batch(_deps, _info, from, ids, amounts), 
    }
}

pub mod execute {
    use super::*;

    pub fn set_approval_for_all(
        deps: DepsMut, 
        info: MessageInfo, 
        operator: Addr, 
        approved: bool
    ) -> Result<Response, ContractError> {
        
        if info.sender == operator {
            return Err(ContractError::SelfApprovedError {});
        }

        let approved_function = |approve: Option<bool>| -> StdResult<bool> {
            match approve {
                Some(_) => Ok(approved),
                None => Ok(approved)
            }
        };

        OPERATOR_APPROVAL.update(deps.storage, (&info.sender, &operator), approved_function)?;
        Ok(Response::new().add_attribute("method", "set_approval_for_all"))
    }

    pub fn safe_transfer_from(
        deps: DepsMut, 
        _info: MessageInfo,
        from: Addr, 
        to: Addr, 
        id: u64, 
        amount: u64
    ) -> Result<Response, ContractError> {
        let zero_address = Addr::unchecked("");

        if from == zero_address || to == zero_address {
            return Err(ContractError::InvalidAddress {});
        }

        let (id_array, amount_array) = change_to_array(id, amount);

        update(deps, from, to, &id_array, &amount_array)?;
        Ok(Response::new().add_attribute("method", "safe_transfer_from"))
    }

    pub fn safe_batch_transfer_from(
        deps: DepsMut, 
        _info: MessageInfo,
        from: Addr,
        to: Addr,
        ids: Vec<u64>,
        amounts: Vec<u64>
    ) -> Result<Response, ContractError> {
        let zero_address = Addr::unchecked("");

        if from == zero_address || to == zero_address {
            return Err(ContractError::InvalidAddress {});
        }

        update(deps, from, to, &ids, &amounts)?;
        Ok(Response::new().add_attribute("method", "safe_batch_transfer_from"))
    }

    pub fn mint(
        deps: DepsMut, 
        _info: MessageInfo,
        to: Addr,
        id: u64,
        amount: u64
    ) -> Result<Response, ContractError> {
        let is_valid = deps.api.addr_validate(&to.as_str());

        if let Err(_error) = is_valid {
            return Err(ContractError::InvalidAddress {})
        }

        let (id_array, amount_array) = change_to_array(id, amount);
        let zero_address = Addr::unchecked("");

        update(deps, zero_address, to, &id_array, &amount_array)?;
        Ok(Response::new().add_attribute("method", "mint"))
    }

    pub fn mint_batch(
        deps: DepsMut, 
        _info: MessageInfo,
        to: Addr,
        ids: Vec<u64>,
        amounts: Vec<u64>
    ) -> Result<Response, ContractError> {
        let is_valid = deps.api.addr_validate(&to.as_str());

        if let Err(_error) = is_valid {
            return Err(ContractError::InvalidAddress {})
        }

        let zero_address = Addr::unchecked("");

        update(deps, zero_address, to, &ids, &amounts)?;
        Ok(Response::new().add_attribute("method", "mint_batch"))      
    }

    pub fn burn(
        deps: DepsMut, 
        _info: MessageInfo,
        from: Addr,
        id: u64,
        amount: u64
    ) -> Result<Response, ContractError> {
        let is_valid = deps.api.addr_validate(&from.as_str());

        if let Err(_error) = is_valid {
            return Err(ContractError::InvalidAddress {})
        }

        let (id_array, amount_array) = change_to_array(id, amount);
        let zero_address = Addr::unchecked("");

        update(deps, from, zero_address, &id_array, &amount_array)?;
        Ok(Response::new().add_attribute("method", "burn"))
    }

    pub fn burn_batch(
        deps: DepsMut, 
        _info: MessageInfo,
        from: Addr,
        ids: Vec<u64>,
        amounts: Vec<u64>
    ) -> Result<Response, ContractError> {
        let is_valid = deps.api.addr_validate(&from.as_str());

        if let Err(_error) = is_valid {
            return Err(ContractError::InvalidAddress {})
        }
        
        let zero_address = Addr::unchecked("");

        update(deps, from, zero_address, &ids, &amounts)?;
        Ok(Response::new().add_attribute("method", "burn_batch"))
    }

    pub fn change_to_array(id: u64, amount: u64) -> ([u64; 1], [u64; 1]) {
        let id_array = [id];
        let amount_array = [amount];

        (id_array, amount_array)
    }

    pub fn update(deps: DepsMut, from: Addr, to: Addr, ids: &[u64], amounts: &[u64]) -> Result<Response, ContractError> {
        if ids.len() != amounts.len() {
            return Err(ContractError::InvalidIdAmountLength {});
        } 

        for i in 0..ids.len() {
            let id = ids[i];
            let amount = amounts[i];
            let zero_address = Addr::unchecked("");

            if zero_address != from {
                let from_balance = BALANCES.load(deps.storage, (&id.to_string(), &from.as_str()))?;

                if from_balance < amount {
                    return Err(ContractError::InsufficientBalance {});
                }

                let balance_update = |balance| -> StdResult<u64>{
                    match balance {
                        Some(bal) => Ok(bal - amount),
                        None => Ok(0),
                    }
                };

                BALANCES.update(deps.storage, (&id.to_string(), &from.as_str()), balance_update)?;
            }

            if zero_address != to {
                let balance_update = |balance| -> StdResult<u64>{
                    match balance {
                        Some(bal) => Ok(bal + amount),
                        None => Ok(amount),
                    }
                };

                BALANCES.update(deps.storage, (&id.to_string(), &to.as_str()), balance_update)?;
            }
        }

        Ok(Response::new().add_attribute("method", "update"))
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(
    _deps: Deps, 
    _env: Env, 
    _msg: QueryMsg
) -> StdResult<Binary> {
    match _msg {
        QueryMsg::BalanceOf { account, id } => to_binary(&query::balance_of(_deps, account, id)?),
        QueryMsg::BalanceOfBatch { accounts, ids } => to_binary(&query::balance_of_batch(_deps, accounts, ids)?),
        QueryMsg::IsApprovedForAll { account, operator } => to_binary(&query::is_approved_for_all(_deps, account, operator)?),
    }
}

pub mod query {
    use super::*;

    pub fn balance_of(deps: Deps, account: Addr, id: u64) -> StdResult<BalanceOfResponse> {

        let balance = BALANCES.load(deps.storage, (&id.to_string(), &account.as_str()))?;

        Ok(BalanceOfResponse { balanceOf: balance })
    }

    pub fn balance_of_batch(deps: Deps, account: Vec<Addr>, ids: Vec<u64>) -> StdResult<BalanceOfBatchResponse> {
        let mut balance_batch: Vec<u64> = Vec::new();

        for i in 0..ids.len() {
            balance_batch.push(BALANCES.load(deps.storage, (&ids[i].to_string(), &account[i].as_str()))?);
        }

        Ok(BalanceOfBatchResponse { balanceOfBatch: balance_batch })
    }

    pub fn is_approved_for_all(deps: Deps, account: Addr, operator: Addr) -> StdResult<IsApprovedForAllResponse> {
        Ok(IsApprovedForAllResponse { isApprovedForAll: OPERATOR_APPROVAL.load(deps.storage, (&account, &operator)).unwrap()})
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::{coins, from_binary };

    #[test]
    fn proper_initialization() {
        let mut deps = mock_dependencies();
        let info = mock_info("owner", &coins(1000, "earth"));
        let msg = InstantiateMsg {};
        let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());
    }

    #[test]
    fn mint_burn_and_query() {
        // instantiate contract 

        let mut deps = mock_dependencies();
        let info = mock_info("owner", &coins(1000, "earth"));
        let msg = InstantiateMsg {};
        let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());

        // mint token

        let owner = mock_info("owner", &coins(1000, "earth"));
        let user_one = Addr::unchecked("user_one");
        let msg = ExecuteMsg::Mint { to: user_one, id: 1, amount: 10 };
        let _res = execute(deps.as_mut(), mock_env(), owner, msg);

        // query mint token
        
        let user_one = Addr::unchecked("user_one");
        let msg = QueryMsg::BalanceOf { account: user_one, id: 1 };
        let res = query(deps.as_ref(), mock_env(), msg).unwrap();

        let value: BalanceOfResponse = from_binary(&res).unwrap();
        assert_eq!(10, value.balanceOf);

        // burn token 

        let owner = mock_info("owner", &coins(1000, "earth"));
        let user_one = Addr::unchecked("user_one");
        let msg = ExecuteMsg::Burn { from: user_one, id: 1, amount: 5 };
        let _res = execute(deps.as_mut(), mock_env(), owner, msg);

        // query burn token 

        let user_one = Addr::unchecked("user_one");
        let msg = QueryMsg::BalanceOf { account: user_one, id: 1 };
        let res = query(deps.as_ref(), mock_env(), msg).unwrap();

        let value: BalanceOfResponse = from_binary(&res).unwrap();
        assert_eq!(5, value.balanceOf);    

        // safe transfer from 
        let owner = mock_info("owner", &coins(1000, "earth"));

        let user_one = Addr::unchecked("user_one");
        let user_two = Addr::unchecked("user_two");

        let msg = ExecuteMsg::SafeTransferFrom { from: user_one, to: user_two, id: 1, amount: 1 };

        let _res = execute(deps.as_mut(), mock_env(), owner, msg);     

        // query token
        
        let user_two = Addr::unchecked("user_two");
        let msg = QueryMsg::BalanceOf { account: user_two, id: 1 };
        let res = query(deps.as_ref(), mock_env(), msg).unwrap();

        let value: BalanceOfResponse = from_binary(&res).unwrap();
        assert_eq!(1, value.balanceOf);

    }

    #[test]
    fn mint_batch_burn_batch_and_query() {
        // instantiate contract 

        let mut deps = mock_dependencies();
        let info = mock_info("owner", &coins(1000, "earth"));
        let msg = InstantiateMsg {};
        let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());

        // mint batch token

        let owner = mock_info("owner", &coins(1000, "earth"));
        let user_one = Addr::unchecked("user_one");
        
        let mut id_array: Vec<u64> = Vec::new(); id_array.push(1); id_array.push(2); id_array.push(3); id_array.push(4);
        let mut amount_array: Vec<u64> = Vec::new(); amount_array.push(10); amount_array.push(10); amount_array.push(10); amount_array.push(10);

        let msg = ExecuteMsg::MintBatch { to: user_one, ids: id_array, amounts: amount_array };
        let _res = execute(deps.as_mut(), mock_env(), owner, msg);

        // query batch token

        let user_one = Addr::unchecked("user_one");

        let mut id_array: Vec<u64> = Vec::new(); id_array.push(1); 
        let mut accounts_array = Vec::new(); accounts_array.push(user_one);

        let msg = QueryMsg::BalanceOfBatch { accounts: accounts_array, ids: id_array };
        let res = query(deps.as_ref(), mock_env(), msg).unwrap();

        let value: BalanceOfBatchResponse = from_binary(&res).unwrap();
        assert_eq!(10, value.balanceOfBatch[0]);

        // burn batch token

        let owner = mock_info("owner", &coins(1000, "earth"));
        let user_one = Addr::unchecked("user_one");

        let mut id_array: Vec<u64> = Vec::new(); id_array.push(1); id_array.push(2); id_array.push(3); id_array.push(4);
        let mut amount_array: Vec<u64> = Vec::new(); amount_array.push(5); amount_array.push(5); amount_array.push(5); amount_array.push(5);

        let msg = ExecuteMsg::BurnBatch { from: user_one, ids: id_array, amounts: amount_array };
        let _res = execute(deps.as_mut(), mock_env(), owner, msg);

        // query batch token 

        let user_one = Addr::unchecked("user_one");

        let mut id_array: Vec<u64> = Vec::new(); id_array.push(1); 
        let mut accounts_array = Vec::new(); accounts_array.push(user_one);

        let msg = QueryMsg::BalanceOfBatch { accounts: accounts_array, ids: id_array };
        let res = query(deps.as_ref(), mock_env(), msg).unwrap();

        let value: BalanceOfBatchResponse = from_binary(&res).unwrap();
        assert_eq!(5, value.balanceOfBatch[0]);

        // safe batch transfer from 

        let owner = mock_info("owner", &coins(1000, "earth"));
        let user_one = Addr::unchecked("user_one");
        let user_two = Addr::unchecked("user_two");

        let mut id_array: Vec<u64> = Vec::new(); id_array.push(1); id_array.push(2); id_array.push(3); id_array.push(4);
        let mut amount_array: Vec<u64> = Vec::new(); amount_array.push(1); amount_array.push(1); amount_array.push(1); amount_array.push(1);  

        let msg = ExecuteMsg::SafeBatchTransferFrom { from: user_one, to: user_two, ids: id_array, amounts: amount_array };

        let _res = execute(deps.as_mut(), mock_env(), owner, msg);

        // query batch token    

        let user_two = Addr::unchecked("user_two");

        let mut id_array: Vec<u64> = Vec::new(); id_array.push(1); 
        let mut accounts_array = Vec::new(); accounts_array.push(user_two);

        let msg = QueryMsg::BalanceOfBatch { accounts: accounts_array, ids: id_array };
        let res = query(deps.as_ref(), mock_env(), msg).unwrap();

        let value: BalanceOfBatchResponse = from_binary(&res).unwrap();
        assert_eq!(1, value.balanceOfBatch[0]);  

    }

    #[test]
    fn set_approval_for_all() {
        // instantiate contract 

        let mut deps = mock_dependencies();
        let info = mock_info("owner", &coins(1000, "earth"));
        let msg = InstantiateMsg {};
        let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());

        // set approval for all 

        let owner = mock_info("owner", &coins(1000, "earth"));
        let user_one = Addr::unchecked("user_one");

        let msg = ExecuteMsg::SetApprovalForAll { operator: user_one, approved: true };
        let _res = execute(deps.as_mut(), mock_env(), owner, msg);

        // query is approval true

        let owner = mock_info("owner", &coins(1000, "earth"));
        let user_one = Addr::unchecked("user_one");

        let msg = QueryMsg::IsApprovedForAll { account: owner.sender, operator: user_one };
        let res = query(deps.as_ref(), mock_env(), msg).unwrap();
        let value: IsApprovedForAllResponse = from_binary(&res).unwrap();

        assert_eq!(true, value.isApprovedForAll);

    }
}