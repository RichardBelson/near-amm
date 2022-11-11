/*
 * Example smart contract written in RUST
 *
 * Learn more about writing NEAR smart contracts with Rust:
 * https://near-docs.io/develop/Contract
 *
 */

use near_contract_standards::fungible_token::core::ext_ft_core;
use near_contract_standards::fungible_token::metadata::{ext_ft_metadata, FungibleTokenMetadata};
use near_contract_standards::fungible_token::receiver::FungibleTokenReceiver;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::env::panic_str;
use near_sdk::json_types::U128;
use near_sdk::serde_json::json;
use near_sdk::{
    env, near_bindgen, AccountId, Balance, Gas, PanicOnDefault, Promise, PromiseError,
    PromiseOrValue, PromiseResult,
};

pub const TGAS: u64 = 1_000_000_000_000;
pub const FIXED_GAS: Gas = Gas(5 * TGAS);

// Define the contract structure
#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct AMMContract {
    owner_id: AccountId,
    ft_a: FTToken,
    ft_b: FTToken,
    ratio: Balance,
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
struct FTToken {
    account_id: AccountId,
    name: String,
    decimals: u8,
    balance: Balance,
}

// Implement the contract structure
#[near_bindgen]
impl AMMContract {
    #[init]
    pub fn init(owner_id: AccountId, ft_a_id: AccountId, ft_b_id: AccountId) -> Self {
        Self::get_ft_metadata(ft_a_id.clone())
            .and(Self::get_ft_metadata(ft_b_id.clone()))
            .then(
                Self::ext(env::current_account_id())
                    .with_static_gas(FIXED_GAS)
                    .query_metadata_callback(),
            );

        Self {
            owner_id,
            ft_a: FTToken {
                account_id: ft_a_id.clone(),
                name: "".to_string(),
                decimals: 0,
                balance: 0,
            },
            ft_b: FTToken {
                account_id: ft_b_id.clone(),
                name: "".to_string(),
                decimals: 0,
                balance: 0,
            },
            ratio: 0,
        }
    }

    fn get_ft_metadata(ft_account: AccountId) -> Promise {
        ext_ft_metadata::ext(ft_account)
            .with_static_gas(FIXED_GAS)
            .ft_metadata()
    }

    fn update_ratio(&mut self) {
        self.ratio = self.ft_a.balance.checked_mul(self.ft_b.balance).unwrap();
    }

    pub fn deposit_a(&mut self, sender_id: AccountId, amount: Balance) {
        if sender_id == self.owner_id {
            self.deposit_a_by_owner(amount);
            return;
        }
        let new_a_balance = self.ft_a.balance.checked_add(amount.into()).unwrap();
        let new_b_balance = self.ratio / new_a_balance;
        ext_ft_core::ext(self.ft_b.account_id.clone())
            .with_static_gas(FIXED_GAS)
            .with_attached_deposit(1)
            .ft_transfer(sender_id, (self.ft_b.balance - new_b_balance).into(), None)
            .then(
                Self::ext(env::current_account_id())
                    .with_static_gas(FIXED_GAS)
                    .ft_transfer_callback(new_a_balance, new_b_balance),
            );
    }

    fn deposit_a_by_owner(&mut self, amount: Balance) {
        self.ft_a.balance = self.ft_a.balance.checked_add(amount.into()).unwrap();
        self.update_ratio();
    }

    fn deposit_b(&mut self, sender_id: AccountId, amount: Balance) {
        if sender_id == self.owner_id {
            self.deposit_b_by_owner(amount);
            return;
        }
        let new_b_balance = self.ft_b.balance.checked_add(amount.into()).unwrap();
        let new_a_balance = self.ratio / self.ft_b.balance;
        ext_ft_core::ext(self.ft_b.account_id.clone())
            .with_static_gas(FIXED_GAS)
            .with_attached_deposit(1)
            .ft_transfer(sender_id, (self.ft_a.balance - new_a_balance).into(), None)
            .then(
                Self::ext(env::current_account_id())
                    .with_static_gas(FIXED_GAS)
                    .ft_transfer_callback(new_a_balance, new_b_balance),
            );
    }

    fn deposit_b_by_owner(&mut self, amount: Balance) {
        self.ft_b.balance = self.ft_b.balance.checked_add(amount.into()).unwrap();
        self.update_ratio();
    }

    #[private]
    pub fn query_metadata_callback(
        &mut self,
        #[callback_result] a_metadata_result: Result<FungibleTokenMetadata, PromiseError>,
        #[callback_result] b_metadata_result: Result<FungibleTokenMetadata, PromiseError>,
    ) {
        let a_meta = a_metadata_result.unwrap();
        self.ft_a.name = a_meta.name;
        self.ft_a.decimals = a_meta.decimals;
        let b_meta = b_metadata_result.unwrap();
        self.ft_b.name = b_meta.name;
        self.ft_b.decimals = b_meta.decimals;
    }

    #[private]
    pub fn ft_transfer_callback(&mut self, a_balance: Balance, b_balance: Balance) {
        match env::promise_result(0) {
            PromiseResult::NotReady => panic_str("not ready"),
            PromiseResult::Failed => panic_str("can't buy b"),
            PromiseResult::Successful(_) => {
                self.ft_a.balance = a_balance;
                self.ft_b.balance = b_balance;
                self.update_ratio();
            }
        }
    }

    pub fn get_tokens_info(&self) -> String {
        json!({
            "a": json!({
                "name":         self.ft_a.name,
                "account_id":   self.ft_a.account_id,
                "desimals":     self.ft_a.decimals,
                "balance":       U128::from(self.ft_a.balance),
            }),
            "b": json!({
                "name":         self.ft_b.name,
                "account_id":   self.ft_b.account_id,
                "decimals":     self.ft_b.decimals,
                "balance":       U128::from(self.ft_b.balance),
            })
        })
        .to_string()
    }

    pub fn get_ratio(&self) -> Balance {
        self.ratio
    }
}

#[near_bindgen]
impl FungibleTokenReceiver for AMMContract {
    #[allow(unused_variables)]
    fn ft_on_transfer(
        &mut self,
        sender_id: AccountId,
        amount: U128,
        msg: String,
    ) -> PromiseOrValue<U128> {
        let token_id = env::predecessor_account_id();
        if token_id != self.ft_a.account_id && token_id != self.ft_b.account_id {
            env::panic_str("unsuport fungible token")
        }
        if token_id == self.ft_a.account_id {
            self.deposit_a(sender_id, amount.into());
        } else {
            self.deposit_b(sender_id, amount.into());
        }
        return PromiseOrValue::Value(0.into());
    }
}
