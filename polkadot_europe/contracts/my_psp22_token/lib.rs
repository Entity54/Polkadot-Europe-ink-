#![cfg_attr(not(feature = "std"), no_std)]
#![feature(min_specialization)]

#[openbrush::contract]
mod my_psp22_token {

    use ink_storage::traits::SpreadAllocate;

    // use openbrush::contracts::psp22::*;
    use openbrush::contracts::ownable::*;
    use openbrush::contracts::psp22::extensions::metadata::*;
    use openbrush::contracts::psp22::extensions::mintable::*;
    use openbrush::traits::Storage;
    use openbrush::traits::String;

    #[ink(storage)]
    #[derive(Default, SpreadAllocate, Storage)]
    pub struct MyPsp22Token {
        #[storage_field]
        psp22: psp22::Data,
        #[storage_field]
        ownable: ownable::Data,
        #[storage_field]
        metadata: metadata::Data,
    }

    impl PSP22 for MyPsp22Token {}
    impl Ownable for MyPsp22Token {}
    impl PSP22Mintable for MyPsp22Token {
        #[ink(message)]
        #[openbrush::modifiers(only_owner)]
        fn mint(&mut self, account: AccountId, amount: Balance) -> Result<(), PSP22Error> {
            self._mint_to(account, amount)
        }
    }

    impl PSP22Metadata for MyPsp22Token {}

    impl MyPsp22Token {
        #[ink(constructor)]
        pub fn new(
            initial_supply: Balance,
            name: Option<String>,
            symbol: Option<String>,
            decimal: u8,
        ) -> Self {
            ink_lang::codegen::initialize_contract(|_instance: &mut Self| {
                _instance
                    ._mint_to(_instance.env().caller(), initial_supply)
                    .expect("Should mint");
                _instance._init_with_owner(_instance.env().caller());
                _instance.metadata.name = name;
                _instance.metadata.symbol = symbol;
                _instance.metadata.decimals = decimal;
            })
        }
    }
}
