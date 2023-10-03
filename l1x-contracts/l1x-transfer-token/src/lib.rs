use l1x_sdk::contract;

use l1x_sdk::types::Address;
use l1x_sdk::types::U128;

pub struct Contract {}

#[contract]
impl Contract {
    pub fn new() {}

    pub fn fund_contract(amount: U128) {
        // Transfer `amount` from the caller to the contract's address
        l1x_sdk::transfer_from_caller(amount.0);
    }

    pub fn transfer(to: Address, amount: U128) {
        // Transfer `amount` from the contract's address to `to`
        l1x_sdk::transfer_to(&to, amount.0)
    }

    pub fn caller_balance() -> U128 {
        l1x_sdk::address_balance(&l1x_sdk::caller_address()).into()
    }

    pub fn contract_balance() -> U128 {
        l1x_sdk::address_balance(&l1x_sdk::contract_instance_address()).into()
    }
}
