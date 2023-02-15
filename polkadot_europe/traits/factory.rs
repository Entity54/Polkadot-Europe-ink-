use ink_prelude::{vec, vec::Vec};

use openbrush::{
    contracts::traits::{access_control::*, psp22::*},
    traits::{AccountId, Balance, String},
};

#[openbrush::wrapper]
pub type FactoryRef = dyn Factory;

#[openbrush::trait_definition]
pub trait Factory {
    //

    #[ink(message, payable)]
    fn launch_treasury_manager(
        &mut self,
        contract_administrator: AccountId,
        contract_manager: AccountId,
        treasury_token_symbol: String,
        treasury_token_address: AccountId,
        usdt_token_address: AccountId,
        oracle_dex_address: AccountId,
        liabilities_threshold_level: u8,
    );

    #[ink(message)]
    fn get_owners(&self) -> Vec<AccountId>;

    #[ink(message)]
    fn get_owner_contract_address(&self, owner: AccountId) -> AccountId;
}
