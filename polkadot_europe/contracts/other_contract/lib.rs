#![cfg_attr(not(feature = "std"), no_std)]

use ink_lang as ink;

pub use self::other_contract::{OtherContract, OtherContractRef};

#[ink::contract]
pub mod other_contract {

    use ink_prelude::{vec, vec::Vec};
    use ink_storage::traits::{PackedLayout, SpreadLayout};
    use ink_storage::traits::{SpreadAllocate, StorageLayout};

    /// Storage for the other contract.
    #[ink(storage)]
    #[derive(Default, SpreadAllocate)]
    pub struct OtherContract {
        value: i32,
    }

    // impl Default for OtherContract {}
    impl OtherContract {
        /// Initializes the contract.
        #[ink(constructor)]
        pub fn new(value: i32) -> Self {
            Self { value }
        }

        /// Returns the current state.
        #[ink(message)]
        pub fn get_value(&self) -> i32 {
            self.value
        }

        /// Mutates the internal value.
        #[ink(message)]
        pub fn inc(&mut self, by: i32) {
            self.value += by;
        }
    }
}
