#![cfg_attr(not(feature = "std"), no_std)]
#![feature(min_specialization)]

#[openbrush::contract]
mod oracle {

    use ink_lang::codegen::Env;
    use polkadot_europe::traits::oracle_dex::*;

    use ink_prelude::{vec, vec::Vec};
    use ink_storage::traits::{PackedLayout, SpreadLayout};
    use ink_storage::traits::{SpreadAllocate, StorageLayout};
    use openbrush::{
        contracts::{access_control::*, traits::errors::PSP22Error, traits::psp22::PSP22Ref},
        modifiers,
        storage::Mapping,
        traits::{Storage, String},
    };

    #[ink(storage)]
    #[derive(Default, SpreadAllocate, Storage)]
    pub struct Oracle {
        #[storage_field]
        access: access_control::Data,
        pair_price: Mapping<(AccountId, AccountId), Balance>,
        pair_state: Mapping<(AccountId, AccountId), bool>,
        pool_liquidity: Mapping<AccountId, Balance>,
        pool_state: Mapping<AccountId, bool>,

        average_length: u8,
        average_prices_pointer: Mapping<(AccountId, AccountId), u8>,
        average_prices_constituents: Mapping<(AccountId, AccountId), Vec<Balance>>,
        average_prices: Mapping<(AccountId, AccountId), Balance>,
    }

    const ADMIN: RoleType = ink_lang::selector_id!("ADMIN");

    impl OracleDex for Oracle {
        #[ink(message)]
        #[modifiers(only_role(ADMIN))]
        fn update_price(
            &mut self,
            base_token: AccountId,
            quote_token: AccountId,
            price: Balance,
        ) -> Result<(), AccessControlError> {
            if self.get_pair_state(base_token, quote_token) {
                self.pair_price.insert(&(base_token, quote_token), &price);
                ink_env::debug_println!(
                    "update_price: base: {:?} quote: {:?} price: {:?} submitted: {:?}",
                    &base_token,
                    &quote_token,
                    &price,
                    self.get_pair_state(base_token, quote_token)
                );
            }
            Ok(())
        }

        #[ink(message)]
        fn get_average_length(&self) -> u8 {
            self.average_length
        }

        // #[ink(message)]
        fn get_average_prices_pointer(&self, base_token: AccountId, quote_token: AccountId) -> u8 {
            match self.average_prices_pointer.get(&(base_token, quote_token)) {
                Some(val) => val,
                None => 0,
            }
        }

        #[ink(message)]
        fn get_average_price(&self, base_token: AccountId, quote_token: AccountId) -> Balance {
            match self.average_prices.get(&(base_token, quote_token)) {
                Some(val) => val,
                None => 0,
            }
        }

        #[ink(message)]
        fn get_average_prices_constituents(
            &self,
            base_token: AccountId,
            quote_token: AccountId,
        ) -> Vec<Balance> {
            match self
                .average_prices_constituents
                .get(&(base_token, quote_token))
            {
                Some(valvector) => valvector.clone(),
                None => Default::default(),
            }
        }

        //no need for  #[ink(message)] just for testing
        // #[ink(message)]
        fn set_average_prices_pointer(
            &mut self,
            base_token: AccountId,
            quote_token: AccountId,
            new_pointer: u8,
        ) {
            self.average_prices_pointer
                .insert(&(base_token, quote_token), &new_pointer);
        }

        #[ink(message)]
        #[modifiers(only_role(ADMIN))]
        fn update_average_price(
            &mut self,
            base_token: AccountId,
            quote_token: AccountId,
            price: Balance,
        ) -> Result<(), AccessControlError> {
            if self.get_pair_state(base_token, quote_token) {
                //
                let vector_prices_length: u8 = match self
                    .average_prices_constituents
                    .get(&(base_token, quote_token))
                {
                    Some(valvector) => valvector.len() as u8,
                    None => 0,
                };

                let mut pointer = self.get_average_prices_pointer(base_token, quote_token);
                let mut vector_prices =
                    self.get_average_prices_constituents(base_token, quote_token);

                ink_env::debug_println!(
                    "vector_prices_length: {:?} pointer: {:?} vector_prices: {:?}",
                    &vector_prices_length,
                    &pointer,
                    &vector_prices,
                );

                if vector_prices_length < (self.average_length + 1) {
                    if vector_prices_length == 0 {
                        vector_prices.push(price);
                        vector_prices.push(price); //storing sum at last element
                    } else {
                        let sum = vector_prices[(vector_prices_length - 1) as usize];
                        vector_prices[(vector_prices_length - 1) as usize] = price;
                        vector_prices.push(sum + price);
                    }

                    ink_env::debug_println!(
                        "A> vector_prices_length: {:?} pointer: {:?} vector_prices: {:?}",
                        &vector_prices_length,
                        &pointer,
                        &vector_prices,
                    );
                } else {
                    let outgoing_element = vector_prices[pointer as usize];
                    //new sum of elements
                    let sum =
                        vector_prices[self.average_length as usize] + price - outgoing_element;
                    vector_prices[self.average_length as usize] = sum;
                    vector_prices[pointer as usize] = price;

                    if pointer < (self.average_length - 1) {
                        pointer += 1;
                    } else {
                        pointer = 0;
                    }
                    self.set_average_prices_pointer(base_token, quote_token, pointer);
                    let avg_price = sum / (self.average_length as u128);
                    self.average_prices
                        .insert(&(base_token, quote_token), &avg_price);

                    ink_env::debug_println!(
                        "B> vector_prices_length: {:?} pointer: {:?} vector_prices: {:?} avg_price: {:?}",
                        &vector_prices_length,
                        &pointer,
                        &vector_prices,
                        &avg_price
                    );
                    ink_env::debug_println!(
                        "C> update_average_price: pointer: {:?} average_prices_constituents: {:?} get_average_price: {:?}",
                        &self.get_average_prices_pointer(base_token, quote_token),
                        &self.get_average_prices_constituents(base_token, quote_token),
                        &self.get_average_price(base_token, quote_token),
                    );
                }

                self.average_prices_constituents
                    .insert(&(base_token, quote_token), &vector_prices);

                ink_env::debug_println!(
                        "FINAL update_average_price: pointer: {:?} average_prices_constituents: {:?} get_average_price: {:?}",
                        &self.get_average_prices_pointer(base_token, quote_token),
                        &self.get_average_prices_constituents(base_token, quote_token),
                        &self.get_average_price(base_token, quote_token),
                    );

                self.update_price(base_token, quote_token, price);
            }
            Ok(())
        }

        #[ink(message)]
        fn get_pair_price(&self, base_token: AccountId, quote_token: AccountId) -> Balance {
            match self.pair_price.get(&(base_token, quote_token)) {
                Some(val) => val,
                None => 0,
            }
        }

        #[ink(message)]
        fn get_pair_state(&self, base_token: AccountId, quote_token: AccountId) -> bool {
            match self.pair_state.get(&(base_token, quote_token)) {
                Some(val) => val,
                None => false,
            }
        }

        #[ink(message)]
        fn get_pool_liquidity(&self, token_address: AccountId) -> Balance {
            match self.pool_liquidity.get(&token_address) {
                Some(val) => val,
                None => 0,
            }
        }

        #[ink(message)]
        fn get_pool_state(&self, token_address: AccountId) -> bool {
            // self.pool_state.get(&token_address).unwrap()
            // self.pool_state.get(&token_address)
            match self.pool_state.get(&token_address) {
                Some(val) => true,
                None => false,
            }
        }

        #[ink(message)]
        #[modifiers(only_role(ADMIN))]
        fn activate_pair(
            &mut self,
            base_token: AccountId,
            quote_token: AccountId,
        ) -> Result<(), AccessControlError> {
            if !self.get_pair_state(base_token, quote_token)
                && self.get_pool_state(base_token)
                && self.get_pool_state(quote_token)
            {
                self.pair_state.insert(&(base_token, quote_token), &true);
            }
            Ok(())
        }

        #[ink(message)]
        #[modifiers(only_role(ADMIN))]
        fn pause_pair(
            &mut self,
            base_token: AccountId,
            quote_token: AccountId,
        ) -> Result<(), AccessControlError> {
            if self.get_pair_state(base_token, quote_token) {
                self.pair_state.insert(&(base_token, quote_token), &false);
            }
            Ok(())
        }

        #[ink(message)]
        #[modifiers(only_role(ADMIN))]
        fn register_pool(&mut self, token: AccountId) -> Result<(), AccessControlError> {
            if !self.get_pool_state(token) {
                self.pool_state.insert(&(token), &true);
            }
            Ok(())
        }

        #[ink(message)]
        #[modifiers(only_role(ADMIN))]
        fn add_pool_liquidity(
            &mut self,
            token: AccountId,
            amount: Balance,
        ) -> Result<(), AccessControlError> {
            if self.get_pool_state(token) {
                //HERE TRANFER FROM REQUIRES THAT ADMIN HAS APPROVED PSP22 TOKEN
                match self.make_deposit(token, amount) {
                    Ok(()) => {
                        self.pool_liquidity
                            .insert((&token), &(self.get_pool_liquidity(token) + amount));
                    }
                    _ => (),
                }
            }
            Ok(())
        }

        #[ink(message)]
        #[modifiers(only_role(ADMIN))]
        fn withdraw_pool_liquidity(
            &mut self,
            token: AccountId,
            amount: Balance,
        ) -> Result<(), AccessControlError> {
            if self.pool_state.get(&token).unwrap() {
                let current_liquidity = self.get_pool_liquidity(token);
                if amount <= current_liquidity {
                    //HERE TRANFER TO ADMIN
                    self.admin_withdrawal(token, amount);
                    self.pool_liquidity
                        .insert((&token), &(current_liquidity - amount));
                }
            }
            Ok(())
        }

        // PSP22Error
        #[ink(message)]
        fn swap(
            &mut self,
            deposited_token: AccountId,
            withdrawn_token: AccountId,
            amount: Balance,
            swap_caller: AccountId,
            use_average_price: bool,
        ) -> Result<Balance, AccessControlError> {
            //
            let mut price: Balance = Default::default();
            let mut withdrawn_amount: Balance = Default::default();

            if self.get_pair_state(deposited_token, withdrawn_token) {
                match use_average_price {
                    true => price = self.get_average_price(deposited_token, withdrawn_token),
                    false => price = self.get_pair_price(deposited_token, withdrawn_token),
                }
                // price = self.get_pair_price(deposited_token, withdrawn_token);
                withdrawn_amount = amount * price;
            } else if self.get_pair_state(withdrawn_token, deposited_token) {
                match use_average_price {
                    true => price = self.get_average_price(withdrawn_token, deposited_token),
                    false => price = self.get_pair_price(withdrawn_token, deposited_token),
                }
                // price = self.get_pair_price(withdrawn_token, deposited_token);
                withdrawn_amount = amount / price;
            } else {
                assert!(1 > 2, "This Pair does not exist");
            }

            if withdrawn_amount > 0 && withdrawn_amount <= self.get_pool_liquidity(withdrawn_token)
            {
                ink_env::debug_println!(
                    "swap: amount: {:?} withdrawn_amount: {:?} liquidity: {:?} caller: {:?}",
                    amount,
                    withdrawn_amount,
                    self.get_pool_liquidity(withdrawn_token),
                    self.env().caller().clone()
                );

                self.make_deposit(deposited_token, amount);

                self.pool_liquidity.insert(
                    (&deposited_token),
                    &(self.get_pool_liquidity(deposited_token) + amount),
                );

                self.pool_liquidity.insert(
                    (&withdrawn_token),
                    &(self.get_pool_liquidity(withdrawn_token) - withdrawn_amount),
                );

                self.approve_token_for_swap(
                    withdrawn_token,
                    self.env().caller().clone(),
                    withdrawn_amount,
                );

                ink_env::debug_println!(
                    "swap: deposited_token: {:?} quote: {:?} price: {:?} submitted: {:?} swap_caller: {:?}",
                    &deposited_token,
                    &withdrawn_token,
                    &price,
                    &withdrawn_amount,
                    &swap_caller,
                );
            }
            Ok(withdrawn_amount)
        }
    }

    impl Oracle {
        #[ink(constructor)]
        pub fn new() -> Self {
            ink_lang::codegen::initialize_contract(|instance: &mut Self| {
                let caller = instance.env().caller();
                instance._init_with_admin(caller);
                instance
                    .grant_role(ADMIN, caller)
                    .expect("Should grant the role");
                instance.pair_price = Default::default();
                instance.pair_state = Default::default();
                instance.pool_liquidity = Default::default();
                instance.pool_state = Default::default();

                instance.average_length = 5;
                instance.average_prices_pointer = Default::default();
                instance.average_prices_constituents = Default::default();
                instance.average_prices = Default::default();
            })
        }

        fn approve_token_for_swap(
            &mut self,
            withdrawn_token: AccountId,
            contract_address: AccountId,
            amount: Balance,
        ) -> Result<(), PSP22Error> {
            PSP22Ref::approve(&withdrawn_token, contract_address, amount)
                .expect("Approval for deposited_token did not go as planned");
            Ok(())
        }

        fn admin_withdrawal(
            &mut self,
            token_address: AccountId,
            amount: Balance,
        ) -> Result<(), AccessControlError> {
            PSP22Ref::transfer(
                &token_address,
                self.env().caller(),
                amount,
                Vec::<u8>::new(),
            )
            .expect("Transfer to ADMIN did not go well");

            //SHOULD EMMIT EVENT

            Ok(())
        }

        fn make_deposit(
            &mut self,
            token_address: AccountId,
            amount: Balance,
        ) -> Result<(), AccessControlError> {
            let from_caller = self.env().caller().clone();
            let contract = self.env().account_id().clone();

            assert!(
                PSP22Ref::allowance(&token_address, from_caller, contract) >= amount,
                "allowance is too low"
            );

            PSP22Ref::transfer_from_builder(
                &token_address,
                from_caller,
                contract,
                amount,
                Vec::<u8>::new(),
            )
            .call_flags(ink_env::CallFlags::default().set_allow_reentry(true))
            .fire()
            .unwrap();

            //SHOULD EMMIT EVENT

            Ok(())
        }
    }
}
