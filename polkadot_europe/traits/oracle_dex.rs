use ink_prelude::{vec, vec::Vec};

use openbrush::{
    contracts::traits::{access_control::*, psp22::*},
    traits::{AccountId, Balance, String},
};

#[openbrush::wrapper]
pub type OracleDexRef = dyn OracleDex;

#[openbrush::trait_definition]
pub trait OracleDex {
    #[ink(message)]
    fn update_price(
        &mut self,
        base_token: AccountId,
        quote_token: AccountId,
        price: Balance,
    ) -> Result<(), AccessControlError>;

    #[ink(message)]
    fn get_pair_price(&self, base_token: AccountId, quote_token: AccountId) -> Balance;

    /// returns if pair state is true (accepting prices) or false
    #[ink(message)]
    fn get_pair_state(&self, base_token: AccountId, quote_token: AccountId) -> bool;

    /// returns liquidity per token
    #[ink(message)]
    fn get_pool_liquidity(&self, token_address: AccountId) -> Balance;

    #[ink(message)]
    fn get_pool_state(&self, token_address: AccountId) -> bool;

    /// Call to change pair state to true from false to ro register it for 1st time so we can interact with it
    #[ink(message)]
    fn activate_pair(
        &mut self,
        base_token: AccountId,
        quote_token: AccountId,
    ) -> Result<(), AccessControlError>;

    /// Pause pair so we cannot interact
    #[ink(message)]
    fn pause_pair(
        &mut self,
        base_token: AccountId,
        quote_token: AccountId,
    ) -> Result<(), AccessControlError>;

    /// rgister pool for new token
    #[ink(message)]
    fn register_pool(&mut self, token: AccountId) -> Result<(), AccessControlError>;

    /// Add liquidity for token. Pool must be already registered
    #[ink(message)]
    fn add_pool_liquidity(
        &mut self,
        token: AccountId,
        amount: Balance,
    ) -> Result<(), AccessControlError>;

    /// Withdraw liquidity from token pool
    #[ink(message)]
    fn withdraw_pool_liquidity(
        &mut self,
        token: AccountId,
        amount: Balance,
    ) -> Result<(), AccessControlError>;

    ///Swap one token for another
    #[ink(message)]
    fn swap(
        &mut self,
        deposited_token: AccountId,
        withdrawn_token: AccountId,
        amount: Balance,
        swap_caller: AccountId,
        use_average_price: bool,
    ) -> Result<Balance, AccessControlError>;

    ///Get length of desired moving average
    #[ink(message)]
    fn get_average_length(&self) -> u8;

    /// show where next pointer is in stored values to calcualate average
    // #[ink(message)]
    fn get_average_prices_pointer(&self, base_token: AccountId, quote_token: AccountId) -> u8;

    // #[ink(message)]
    fn set_average_prices_pointer(
        &mut self,
        base_token: AccountId,
        quote_token: AccountId,
        new_pointer: u8,
    );

    ///Call this to update averag price, it will also update the last price accessible via get_pair_price
    #[ink(message)]
    fn update_average_price(
        &mut self,
        base_token: AccountId,
        quote_token: AccountId,
        price: Balance,
    ) -> Result<(), AccessControlError>;

    ///Returns average price for a pair
    #[ink(message)]
    fn get_average_price(&self, base_token: AccountId, quote_token: AccountId) -> Balance;

    ///Returns the constiutent prices stored for calculating moving average for the average price of the pair
    #[ink(message)]
    fn get_average_prices_constituents(
        &self,
        base_token: AccountId,
        quote_token: AccountId,
    ) -> Vec<Balance>;
}
