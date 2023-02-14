use ink_prelude::{vec, vec::Vec};
use ink_storage::traits::{PackedLayout, SpreadLayout};

use openbrush::{
    contracts::traits::{access_control::*, psp22::*},
    traits::{AccountId, Balance, String},
};

#[cfg(feature = "std")]
use ink_storage::traits::StorageLayout;

#[derive(
    Default, Debug, Clone, scale::Encode, scale::Decode, SpreadLayout, PackedLayout, PartialEq, Eq,
)]
#[cfg_attr(feature = "std", derive(StorageLayout, scale_info::TypeInfo))]
pub enum PaymentType {
    #[default]
    OneOffFutureTime,
    Instalments,
}

#[openbrush::wrapper]
pub type TreasureManagerRef = dyn TreasureManager;

#[openbrush::trait_definition]
pub trait TreasureManager {
    //
    #[ink(message)]
    fn set_manager(&mut self, account: AccountId) -> Result<(), AccessControlError>;

    #[ink(message)]
    fn add_job(
        &mut self,
        title: String,
        applicant: AccountId,
        requested_token: AccountId,
        value_in_usd: bool,
        requested_value: Balance,
        payment_type: PaymentType,
        payment_schedule: Vec<u64>,
        payee_accounts: Vec<AccountId>,
    ) -> Result<(), AccessControlError>;

    #[ink(message)]
    fn remove_job_info(&mut self, id: u32);

    #[ink(message)]
    fn set_check_points_intervals(
        &mut self,
        checkpoint1: u64,
        checkpoint2: u64,
        checkpoint3: u64,
    ) -> Result<(), AccessControlError>;

    #[ink(message)]
    fn admin_withdrawal(&mut self, amount: Balance) -> Result<(), AccessControlError>;

    #[ink(message)]
    fn make_deposit(&mut self, amount: Balance) -> Result<(), AccessControlError>;

    #[ink(message)]
    fn register_foreign_asset(
        &mut self,
        token_symbol: String,
        token_address: AccountId,
    ) -> Result<(), AccessControlError>;

    #[ink(message)]
    fn set_oracle_dex_address(
        &mut self,
        oracle_dex_address: AccountId,
    ) -> Result<(), AccessControlError>;
}
