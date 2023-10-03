use std::collections::BTreeMap;

use borsh::{BorshDeserialize, BorshSerialize};
use l1x_sdk::store::LookupMap;
use l1x_sdk::types::{Address, U128};
use l1x_sdk::{caller_address, contract, contract_owner_address};
use serde::Deserialize;

const STORAGE_CONTRACT_KEY: &[u8; 2] = b"aa";
const STORAGE_BALANCES_KEY: &[u8; 2] = b"ab";
const STORAGE_ALLOWANCES_KEY: &[u8; 2] = b"ac";

#[derive(BorshSerialize, BorshDeserialize, Deserialize)]
pub struct FTMetadata {
    name: String,
    decimals: u8,
    symbol: String,
    icon: Option<String>,
}

#[derive(BorshSerialize, BorshDeserialize, Default, Clone)]
struct FTAllowance {
    spenders: BTreeMap<Address, u128>,
}

impl FTAllowance {
    fn set(&mut self, spender_id: Address, amount: u128) {
        self.spenders.insert(spender_id, amount);
    }

    fn get(&self, spender_id: &Address) -> u128 {
        self.spenders.get(spender_id).cloned().unwrap_or_default()
    }

    fn increase(&mut self, spender_id: &Address, amount: u128) {
        match self.spenders.get_mut(spender_id) {
            Some(current_amount_ref) => *current_amount_ref += amount,
            None => {
                self.spenders.insert(spender_id.clone(), amount);
            }
        };
    }

    fn decrease(&mut self, spender_id: &Address, amount: u128) {
        self.spend(spender_id, amount);
    }

    fn spend(&mut self, spender_id: &Address, amount: u128) {
        match self.spenders.get_mut(spender_id) {
            Some(allowance_amount) => {
                assert!(*allowance_amount > amount, "The allowance is too small");
                *allowance_amount -= amount;
            }
            None => panic!("No allowance for {spender_id}"),
        }
    }
}

enum AllowanceUpdateOp {
    Set,
    Increase,
    Decrease,
    Spend,
}

#[derive(BorshSerialize, BorshDeserialize)]
pub struct L1xFtErc20 {
    metadata: FTMetadata,
    balances: LookupMap<Address, u128>,
    allowances: LookupMap<Address, FTAllowance>,
    total_supply: u128,
}

#[contract]
impl L1xFtErc20 {
    pub fn new(metadata: FTMetadata, account_ids: Vec<Address>, amounts: Vec<U128>) {
        assert_eq!(
            caller_address(),
            contract_owner_address(),
            "Only the owner can call this function"
        );

        let mut contract = Self {
            metadata,
            balances: LookupMap::new(STORAGE_BALANCES_KEY.to_vec()),
            allowances: LookupMap::new(STORAGE_ALLOWANCES_KEY.to_vec()),
            total_supply: Default::default(),
        };
        contract.initialize_balance_holders(account_ids, amounts);
        contract.save();
    }

    fn initialize_balance_holders(&mut self, account_ids: Vec<Address>, amounts: Vec<U128>) {
        assert_eq!(
            account_ids.len(),
            amounts.len(),
            "account_ids and amounts length mismatch"
        );

        assert!(
            self.total_supply == 0,
            "Contract has already been initialized"
        );

        for (account_id, amount) in account_ids.into_iter().zip(amounts) {
            self.balances.insert(account_id, amount.0);
            self.total_supply += amount.0;
        }
    }

    pub fn ft_name() -> String {
        let contract = Self::load();
        contract.metadata.name
    }

    pub fn ft_symbol() -> String {
        let contract = Self::load();
        contract.metadata.symbol
    }

    pub fn ft_decimals() -> u8 {
        let contract = Self::load();
        contract.metadata.decimals
    }

    pub fn ft_mint(recipient_id: Address, amount: U128) {
        assert_eq!(
            caller_address(),
            contract_owner_address(),
            "Only the owner can call this function"
        );

        let mut contract = Self::load();

        contract.mint(&recipient_id, amount.0);

        contract.save();
    }

    pub fn ft_transfer(recipient_id: Address, amount: U128) {
        let mut contract = Self::load();

        let sender_id = l1x_sdk::caller_address();
        contract.transfer(&sender_id, &recipient_id, amount.into());

        contract.save()
    }

    pub fn ft_transfer_from(sender_id: Address, recipient_id: Address, amount: U128) {
        let mut contract = Self::load();
        let spender_id = caller_address();

        contract.allowance_update(AllowanceUpdateOp::Spend, &sender_id, &spender_id, amount.0);
        contract.transfer(&sender_id, &recipient_id, amount.into());

        contract.save();
    }

    pub fn ft_total_supply() -> U128 {
        let contract = Self::load();
        contract.total_supply.into()
    }

    pub fn ft_balance_of(account_id: Address) -> U128 {
        let contract = Self::load();
        contract.balance_of(&account_id).unwrap_or_default().into()
    }

    pub fn ft_approve(spender_id: Address, amount: U128) {
        let mut contract = Self::load();
        let owner_id = caller_address();

        contract.assert_if_no_balance(&owner_id);
        contract.allowance_update(AllowanceUpdateOp::Set, &owner_id, &spender_id, amount.0);

        contract.save();
    }

    pub fn ft_increase_allowance(spender_id: Address, amount: U128) {
        let mut contract = Self::load();
        let owner_id = caller_address();

        contract.assert_if_no_balance(&owner_id);
        contract.allowance_update(
            AllowanceUpdateOp::Increase,
            &owner_id,
            &spender_id,
            amount.0,
        );

        contract.save();
    }

    pub fn ft_decrease_allowance(spender_id: Address, amount: U128) {
        let mut contract = Self::load();
        let owner_id = caller_address();

        contract.assert_if_no_balance(&owner_id);
        contract.allowance_update(
            AllowanceUpdateOp::Decrease,
            &owner_id,
            &spender_id,
            amount.0,
        );

        contract.save();
    }

    pub fn ft_allowance(owner_id: Address, spender_id: Address) -> U128 {
        let contract = Self::load();

        match contract.allowances.get(&owner_id) {
            Some(allowance) => allowance.get(&spender_id).into(),
            None => 0.into(),
        }
    }

    fn mint(&mut self, recipient_id: &Address, amount: u128) {
        let receiver_balance = self.balance_of(&recipient_id).unwrap_or_default();

        let total_supply = self
            .total_supply
            .checked_add(amount)
            .expect("total_supply is overflowed");
        self.total_supply = total_supply;
        self.balances
            .insert(recipient_id.clone(), receiver_balance + amount);

        l1x_sdk::msg(&format!("Minted {} tokens for {}", amount, recipient_id));
    }

    fn transfer(&mut self, sender_id: &Address, recipient_id: &Address, amount: u128) {
        let sender_balance = self.balance_of(&sender_id).unwrap_or_default();
        let receiver_balance = self.balance_of(&recipient_id).unwrap_or_default();
        assert!(sender_balance >= amount, "Not enough balance to transfer");
        self.balances
            .insert(sender_id.clone(), sender_balance - amount);
        self.balances
            .insert(recipient_id.clone(), receiver_balance + amount);
        l1x_sdk::msg(&format!(
            "Transferred {} tokens from {} to {}",
            amount, sender_id, recipient_id
        ));
    }

    fn allowance_update(
        &mut self,
        update_op: AllowanceUpdateOp,
        owner_id: &Address,
        spender_id: &Address,
        amount: u128,
    ) {
        let allowance = self.allowances.get_mut(owner_id);

        match update_op {
            AllowanceUpdateOp::Set => match allowance {
                Some(allowance_ref) => allowance_ref.set(spender_id.clone(), amount),
                None => {
                    let mut new_allowance = FTAllowance::default();
                    new_allowance.set(spender_id.clone(), amount);
                    self.allowances.insert(owner_id.clone(), new_allowance);
                }
            },
            AllowanceUpdateOp::Increase => match allowance {
                Some(allowance_ref) => allowance_ref.increase(spender_id, amount),
                None => {
                    let mut new_allowance = FTAllowance::default();
                    new_allowance.set(spender_id.clone(), amount);
                    self.allowances.insert(owner_id.clone(), new_allowance);
                }
            },
            AllowanceUpdateOp::Decrease => match allowance {
                Some(allowance_ref) => allowance_ref.decrease(spender_id, amount),
                None => panic!("The current allowance is None or zero"),
            },
            AllowanceUpdateOp::Spend => match allowance {
                Some(allowance_ref) => allowance_ref.spend(spender_id, amount),
                None => panic!("{owner_id} didn't set allowance for {spender_id}"),
            },
        }
    }

    fn balance_of(&self, account_id: &Address) -> Option<u128> {
        self.balances.get(account_id).copied()
    }

    fn assert_if_no_balance(&self, account_id: &Address) {
        assert_ne!(
            self.balances.get(account_id),
            None,
            "'{account_id}' should have token on the balance"
        );
    }

    fn load() -> Self {
        match l1x_sdk::storage_read(STORAGE_CONTRACT_KEY) {
            Some(bytes) => Self::try_from_slice(&bytes).unwrap(),
            None => panic!("The contract isn't initialized"),
        }
    }

    fn save(&mut self) {
        l1x_sdk::storage_write(STORAGE_CONTRACT_KEY, &self.try_to_vec().unwrap());
    }
}
