#![cfg_attr(not(feature = "std"), no_std)]
#![feature(min_specialization)]

pub use self::treasury_manager::{TreasuryManager, TreasuryManagerRef};

#[openbrush::contract]
pub mod treasury_manager {

    use ink_lang::codegen::Env;
    use ink_primitives::KeyPtr;

    use polkadot_europe::traits::oracle_dex::*;
    use polkadot_europe::traits::tr_manager::*;

    use ink_prelude::{vec, vec::Vec};
    use ink_storage::traits::{PackedLayout, SpreadLayout};
    use ink_storage::traits::{SpreadAllocate, StorageLayout};
    use openbrush::{
        contracts::{access_control::*, traits::errors::PSP22Error, traits::psp22::PSP22Ref},
        modifiers,
        storage::Mapping,
        traits::{Storage, String},
    };

    #[derive(
        Debug, PartialEq, Eq, scale::Encode, scale::Decode, Clone, SpreadLayout, PackedLayout,
    )]
    #[cfg_attr(feature = "std", derive(StorageLayout, scale_info::TypeInfo))]
    pub enum MoveJobs {
        OpenToPending,
        PendingToCompleted,
    }

    // #[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
    // #[cfg_attr(feature = "std", derive(::scale_info::TypeInfo))]
    // pub enum Error {
    //     CallerIsNotSender,
    //     CallerIsNotRecipient,
    // }

    #[derive(
        Default, Debug, Clone, scale::Encode, scale::Decode, SpreadLayout, PackedLayout, PartialEq,
    )]
    #[cfg_attr(feature = "std", derive(StorageLayout, scale_info::TypeInfo))]
    pub struct JobInfo {
        id: u32,
        title: String,
        hash: String,
        applicant: AccountId,
        requested_token: AccountId,
        value_in_usd: bool,
        requested_value: Balance,
        payment_type: PaymentType,
        payment_schedule: Vec<u64>,
        payee_accounts: Vec<AccountId>,
        next_installment_pointer: u32,
        position_in_vec: u32,
    }

    #[derive(
        Default,
        Debug,
        PartialEq,
        Eq,
        scale::Encode,
        scale::Decode,
        Clone,
        SpreadLayout,
        PackedLayout,
    )]
    #[cfg_attr(feature = "std", derive(StorageLayout, scale_info::TypeInfo))]
    pub enum UpdateJobVecs {
        #[default]
        OpenToPending,
        PendingToCompleted,
    }

    #[ink(event)]
    pub struct liability_threshold_breached_top {
        #[ink(topic)]
        level_type: u8, //0: ALL 2: 2D 7: 7D 30: 30D
        #[ink(topic)]
        current_balance: Balance,
        #[ink(topic)]
        top_up_amount: Balance,
    }

    #[ink(event)]
    pub struct liability_threshold_breached_med {
        #[ink(topic)]
        level_type: u8, //0: ALL 2: 2D 7: 7D 30: 30D
        #[ink(topic)]
        current_balance: Balance,
        #[ink(topic)]
        liability: Balance,
    }

    #[ink(event)]
    pub struct ev_native_payment {
        #[ink(topic)]
        job_id: u32,
        #[ink(topic)]
        to: AccountId,
        #[ink(topic)]
        amount: Balance,
    }

    #[ink(event)]
    pub struct ev_native_usd_payment {
        #[ink(topic)]
        job_id: u32,
        #[ink(topic)]
        to: AccountId,
        #[ink(topic)]
        amount: Balance,
    }

    #[ink(event)]
    pub struct ev_non_native_payment {
        #[ink(topic)]
        job_id: u32,
        #[ink(topic)]
        to: AccountId,
        #[ink(topic)]
        amount: Balance,
    }

    #[ink(storage)]
    #[derive(Default, SpreadAllocate, Storage)]
    pub struct TreasuryManager {
        #[storage_field]
        access: access_control::Data,
        contract_administrator: AccountId,
        contract_manager: AccountId,
        next_id: u32,
        treasury_token_symbol: String,
        treasury_token_address: AccountId,
        jobs: Mapping<u32, JobInfo>,
        open_jobs_ids: Vec<u32>,
        pending_jobs_ids: Vec<u32>,
        completed_jobs_ids: Vec<u32>,
        native_payments_ids: Vec<u32>,
        native_payments_usd_ids: Vec<u32>,
        non_native_payments_ids: Vec<u32>,
        non_native_tokens_vec: Vec<AccountId>,
        oracle_dex_address: AccountId,
        foreign_assets: Mapping<String, AccountId>,
        foreign_assets_vec: Vec<String>,
        check_points_intervals: Vec<u64>,
        liability_in_treasury: Vec<Balance>,
        liability_in_usdt_tokens: Vec<Balance>,
        liability_in_usdt_tokens_treasury: Vec<Balance>,
        liability_health: Vec<u8>,
        liabilities_thresholds: Vec<u8>,
        fake_timestamp: u64,
    }

    impl TreasureManager for TreasuryManager {
        #[ink(message)]
        #[modifiers(only_role(ADMIN))]
        fn set_manager(&mut self, account: AccountId) -> Result<(), AccessControlError> {
            self.renounce_role(MANAGER, self.contract_manager);
            self._setup_role(MANAGER, account);
            self.contract_manager = account;
            Ok(())
        }

        ///Add Job Should be only for ADMIN
        #[ink(message)]
        #[modifiers(only_role(ADMIN))]
        fn add_job(
            &mut self,
            title: String,
            hash: String,
            applicant: AccountId,
            requested_token: AccountId,
            value_in_usd: bool,
            requested_value: Balance,
            payment_type: PaymentType,
            payment_schedule: Vec<u64>,
            payee_accounts: Vec<AccountId>,
        ) -> Result<(), AccessControlError> {
            assert!(
                requested_token == self.treasury_token_address
                    || self.non_native_tokens_vec.contains(&requested_token),
                "requested_token must be registered"
            );

            let mut _value_in_usd = value_in_usd;
            if requested_token != self.treasury_token_address {
                _value_in_usd = false;
            }

            let job = JobInfo {
                id: self.next_id,
                title,
                hash,
                applicant,
                requested_token,
                value_in_usd: _value_in_usd,
                requested_value,
                payment_type,
                payment_schedule,
                payee_accounts,
                next_installment_pointer: 0,
                position_in_vec: self.open_jobs_ids.len() as u32,
            };

            self.jobs.insert(&self.next_id, &job);
            self.open_jobs_ids.push(self.next_id);
            self.next_id += 1;

            Ok(())
        }

        #[ink(message)]
        #[modifiers(only_role(ADMIN))]
        fn remove_job_info(&mut self, id: u32) -> Result<(), AccessControlError> {
            self.jobs.remove(&id);
            Ok(())
        }

        #[ink(message)]
        #[modifiers(only_role(ADMIN))]
        fn admin_withdrawal(&mut self, amount: Balance) -> Result<(), AccessControlError> {
            //APPROVE FIRST
            PSP22Ref::approve(&self.treasury_token_address, self.env().caller(), amount)
                .expect("Approval for ADMIN withdrawing treasury_token did not go well");

            // PSP22Ref::transfer(
            //     &self.treasury_token_address,
            //     self.env().caller(),
            //     amount,
            //     Vec::<u8>::new(),
            // )
            // .expect("Transfer to ADMIN did not go well");

            //SHOULD EMMIT EVENT

            Ok(())
        }

        #[ink(message)]
        #[modifiers(only_role(ADMIN))]
        fn terminate_me(&mut self) -> Result<(), AccessControlError> {
            self.env().terminate_contract(self.env().caller());
            Ok(())
        }

        #[ink(message)]
        #[modifiers(only_role(ADMIN))]
        fn make_deposit(&mut self, amount: Balance) -> Result<(), AccessControlError> {
            let from_caller = self.env().caller().clone();
            let contract = self.env().account_id().clone();

            PSP22Ref::transfer_from_builder(
                &self.treasury_token_address,
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

        //
    }

    #[ink(impl)]
    impl TreasuryManager {
        pub fn swap_in_vecs(&mut self, what_to_swap: MoveJobs, id: u32) {
            let mut origin_vec_ids = &mut Vec::new();
            let mut dest_vec_ids = &mut Vec::new();

            match what_to_swap {
                MoveJobs::OpenToPending => {
                    origin_vec_ids = &mut self.open_jobs_ids;
                    dest_vec_ids = &mut self.pending_jobs_ids;
                }
                MoveJobs::PendingToCompleted => {
                    origin_vec_ids = &mut self.pending_jobs_ids;
                    dest_vec_ids = &mut self.completed_jobs_ids;
                }
            }

            ink_env::debug_println!("1 origin_vec_ids: {:?}", origin_vec_ids);
            ink_env::debug_println!("1 dest_vec_ids: {:?}", dest_vec_ids);

            //get current job with id.
            let current_job: JobInfo = self.jobs.get(&id).unwrap();
            // positing in origin_vec_ids that this job id exists
            let position_in_current_vec = current_job.position_in_vec;
            ink_env::debug_println!("position_in_current_vec: {:?}", position_in_current_vec);

            let origin_vec_size = origin_vec_ids.len() as u32;

            if position_in_current_vec < origin_vec_size {
                //find last elemment in the origin_vec_ids array
                let last_element_job_id = origin_vec_ids[(origin_vec_size - 1) as usize];

                //get the last job
                let mut last_job: JobInfo = self.jobs.get(&last_element_job_id).unwrap().clone();

                last_job = JobInfo {
                    position_in_vec: position_in_current_vec,
                    ..last_job
                };
                ink_env::debug_println!("last_job: {:?}", last_job);

                self.jobs.insert(&last_job.id, &last_job);

                // place the last job id in this positoon. last job has been updated and its ide can now be deleted
                origin_vec_ids[position_in_current_vec as usize] = last_element_job_id;
            }
            //last job id is deleted
            origin_vec_ids.pop();

            let updated_job = JobInfo {
                position_in_vec: dest_vec_ids.len() as u32,
                ..current_job
            };
            ink_env::debug_println!("updated_job: {:?}", updated_job);
            self.jobs.insert(&updated_job.id, &updated_job);

            dest_vec_ids.push(updated_job.id);

            ink_env::debug_println!("2 origin_vec_ids: {:?}", origin_vec_ids);
            ink_env::debug_println!("2 dest_vec_ids: {:?}", dest_vec_ids);
        }
    }

    const ADMIN: RoleType = ink_lang::selector_id!("ADMIN");
    const MANAGER: RoleType = ink_lang::selector_id!("MANAGER");

    impl AccessControl for TreasuryManager {}

    impl TreasuryManager {
        #[ink(constructor)]
        pub fn new(
            contract_administrator: AccountId,
            contract_manager: AccountId,
            treasury_token_symbol: String,
            treasury_token_address: AccountId,
            usdt_token_address: AccountId,
            oracle_dex_address: AccountId,
            liabilities_threshold_level: u8, //10 for 10% leading to 90% and 80% of contract balance
        ) -> Self {
            ink_lang::codegen::initialize_contract(|instance: &mut Self| {
                let caller = instance.env().caller();
                instance._init_with_admin(caller);
                instance
                    .grant_role(ADMIN, contract_administrator)
                    .expect("Should grant the ADMIN role");
                instance
                    .grant_role(MANAGER, contract_manager)
                    .expect("Should grant the MANAGER role");
                instance.contract_administrator = contract_administrator;
                instance.contract_manager = contract_manager;
                instance.next_id = 0;
                instance.treasury_token_symbol = treasury_token_symbol;
                instance.treasury_token_address = treasury_token_address;
                instance.jobs = Default::default();
                instance.open_jobs_ids = Vec::new();
                instance.pending_jobs_ids = Vec::new();
                instance.completed_jobs_ids = Vec::new();
                instance.native_payments_ids = Default::default();
                instance.non_native_payments_ids = Default::default();
                instance.native_payments_usd_ids = Default::default();
                instance.non_native_tokens_vec = vec![usdt_token_address]; //Default::default();
                instance.oracle_dex_address = oracle_dex_address;
                instance
                    .foreign_assets
                    .insert(&String::from("USDT"), &usdt_token_address);
                instance.foreign_assets_vec = vec![String::from("USDT")];
                instance.liabilities_thresholds = vec![
                    (100 - liabilities_threshold_level),
                    (100 - liabilities_threshold_level * 2),
                    (100 + liabilities_threshold_level * 3),
                ];
                instance.check_points_intervals = vec![100, 200, 300];
                instance.liability_in_treasury = vec![0, 0, 0, 0];
                instance.liability_in_usdt_tokens = vec![0, 0, 0, 0];
                instance.liability_in_usdt_tokens_treasury = vec![0, 0, 0, 0];
                instance.liability_health = vec![2, 2, 2, 2];

                instance.fake_timestamp = Default::default();
            })
        }

        //FOR TESTING ONLY TO BE DELETED
        #[ink(message)]
        pub fn get_fake_timestamp(&self) -> u64 {
            self.fake_timestamp
        }
        //FOR TESTING ONLY TO BE DELETED
        #[ink(message)]
        pub fn set_fake_timestamp(&mut self, fresh_timestmap: u64) {
            self.fake_timestamp = fresh_timestmap;
        }
        //FOR TESTING ONLY TO BE DELETED
        #[ink(message)]
        pub fn get_block_timestamp(&self) -> u64 {
            self.env().block_timestamp()
        }

        #[ink(message)]
        pub fn check_open_jobs(&mut self) {
            let mut queued_to_move_job_ids = Vec::new();

            for job_id in &self.open_jobs_ids {
                let current_job: JobInfo = self.jobs.get(&job_id).unwrap();

                // if self.fake_timestamp > current_job.payment_schedule[0]
                if self.env().block_timestamp() > current_job.payment_schedule[0] {
                    queued_to_move_job_ids.push(current_job.id);
                }
            }

            // ink_env::debug_println!("queued_to_move_job_ids: {:?}", queued_to_move_job_ids);

            for job_id in queued_to_move_job_ids {
                self.move_job_from_open_to_pending(job_id);
            }
        }

        #[ink(message)]
        pub fn check_pending_jobs(&mut self) {
            let mut queued_to_move_job_ids = Vec::new();

            for job_id in &self.pending_jobs_ids {
                let mut current_job: JobInfo = self.jobs.get(&job_id).unwrap();

                match current_job.payment_type {
                    PaymentType::OneOffFutureTime => {
                        //categorise and push payment
                        if current_job.requested_token == self.treasury_token_address {
                            if current_job.value_in_usd {
                                self.native_payments_usd_ids.push(current_job.id);
                            } else {
                                self.native_payments_ids.push(current_job.id);
                            }
                        } else {
                            self.non_native_payments_ids.push(current_job.id);
                        }

                        // ink_env::debug_println!(
                        //     "PAYMENT OF PaymentType::OneOffFutureTime id: {:?}",
                        //     current_job.id
                        // );

                        queued_to_move_job_ids.push(current_job.id);
                    }
                    PaymentType::Instalments => {
                        let installment_num = current_job.next_installment_pointer;

                        // if self.fake_timestamp
                        if self.env().block_timestamp()
                            > current_job.payment_schedule[installment_num as usize]
                        {
                            //categorise and push payment
                            if current_job.requested_token == self.treasury_token_address {
                                if current_job.value_in_usd {
                                    self.native_payments_usd_ids.push(current_job.id);
                                } else {
                                    self.native_payments_ids.push(current_job.id);
                                }
                            } else {
                                self.non_native_payments_ids.push(current_job.id);
                            }

                            // ink_env::debug_println!(
                            //     "PAYMENT OF PaymentType::FixedTimeIntervalInstallment id: {:?}",
                            //     current_job.id
                            // );

                            let num_of_payments = current_job.payment_schedule.len() as u32;

                            if num_of_payments > 1 && installment_num < (num_of_payments - 1) {
                                let updated_job = JobInfo {
                                    next_installment_pointer: current_job.next_installment_pointer
                                        + 1,
                                    ..current_job
                                };

                                self.jobs.insert(&updated_job.id, &updated_job);
                            } else {
                                queued_to_move_job_ids.push(current_job.id);
                            }
                        }
                    }
                }
            }

            // ink_env::debug_println!("queued_to_move_job_ids: {:?}", queued_to_move_job_ids);

            for job_id in queued_to_move_job_ids {
                self.move_job_from_pending_to_completed(job_id);
            }
        }

        // #[ink(message)]
        fn move_job_from_open_to_pending(&mut self, id: u32) {
            self.swap_in_vecs(MoveJobs::OpenToPending, id);
        }

        // #[ink(message)]
        fn move_job_from_pending_to_completed(&mut self, id: u32) {
            self.swap_in_vecs(MoveJobs::PendingToCompleted, id);
        }

        #[ink(message)]
        pub fn get_admin_account(&self) -> AccountId {
            self.contract_administrator.clone()
        }

        #[ink(message)]
        pub fn get_manager_account(&self) -> AccountId {
            self.contract_manager.clone()
        }

        #[ink(message)]
        pub fn get_job_info(&self, id: u32) -> JobInfo {
            self.jobs.get(&id).unwrap()
        }

        #[ink(message)]
        pub fn get_next_id(&self) -> u32 {
            self.next_id
        }
        #[ink(message)]
        pub fn get_treasury_token_symbol(&self) -> String {
            self.treasury_token_symbol.clone()
        }
        #[ink(message)]
        pub fn get_treasury_token(&self) -> AccountId {
            self.treasury_token_address.clone()
        }

        #[ink(message)]
        pub fn get_open_jobs_ids(&self) -> Vec<u32> {
            self.open_jobs_ids.clone()
        }
        #[ink(message)]
        pub fn get_pending_jobs_ids(&self) -> Vec<u32> {
            self.pending_jobs_ids.clone()
        }
        #[ink(message)]
        pub fn get_completed_jobs_ids(&self) -> Vec<u32> {
            self.completed_jobs_ids.clone()
        }

        #[ink(message)]
        pub fn get_native_payments_ids(&self) -> Vec<u32> {
            self.native_payments_ids.clone()
        }

        #[ink(message)]
        pub fn get_native_usd_payments_ids(&self) -> Vec<u32> {
            self.native_payments_usd_ids.clone()
        }

        #[ink(message)]
        pub fn get_non_native_payments_ids(&self) -> Vec<u32> {
            self.non_native_payments_ids.clone()
        }

        #[ink(message)]
        pub fn get_non_native_tokens_vec(&self) -> Vec<AccountId> {
            self.non_native_tokens_vec.clone()
        }

        #[ink(message)]
        pub fn get_liability_thresholds(&self) -> Vec<u8> {
            self.liabilities_thresholds.clone()
        }
        #[ink(message)]
        pub fn get_liability_health(&self) -> Vec<u8> {
            self.liability_health.clone()
        }
        #[ink(message)]
        pub fn get_liability_in_usdt_tokens_treasury(&self) -> Vec<Balance> {
            self.liability_in_usdt_tokens_treasury.clone()
        }
        #[ink(message)]
        pub fn get_liability_in_usdt_tokens(&self) -> Vec<Balance> {
            self.liability_in_usdt_tokens.clone()
        }
        #[ink(message)]
        pub fn get_liability_in_treasury(&self) -> Vec<Balance> {
            self.liability_in_treasury.clone()
        }
        #[ink(message)]
        pub fn get_check_points_intervals(&self) -> Vec<u64> {
            self.check_points_intervals.clone()
        }

        #[ink(message)]
        pub fn get_foreign_assets_vec(&self) -> Vec<String> {
            self.foreign_assets_vec.clone()
        }

        #[ink(message)]
        pub fn get_foreign_asset_account(&self, symbol: String) -> AccountId {
            self.foreign_assets.get(&symbol).unwrap_or_default()
        }

        #[ink(message)]
        pub fn get_oracle_dex_address(&self) -> AccountId {
            self.oracle_dex_address.clone()
        }

        //FOR TESTING ONLY AS IT CAN BE FOUND DIRECTLY FROM PSP22
        #[ink(message)]
        pub fn get_balance(&self, token_address: AccountId, account: AccountId) -> Balance {
            PSP22Ref::balance_of(&token_address, account)
        }

        #[ink(message)]
        pub fn make_native_payments(&mut self) -> Result<(), AccessControlError> {
            let mut new_native_payments_ids = Vec::new();

            for job_id in self.native_payments_ids.clone() {
                let current_job: JobInfo = self.jobs.get(&job_id).unwrap();
                let requested_value = current_job.requested_value;
                let payee_accounts = current_job.payee_accounts;

                match self.make_transfer_to(
                    self.treasury_token_address,
                    payee_accounts[0],
                    requested_value,
                ) {
                    Ok(()) => {
                        ink_env::debug_println!(
                            "NATIVE PAYMENT with id: {} has succeeded",
                            current_job.id
                        );

                        self.env().emit_event(ev_native_payment {
                            job_id: job_id,
                            to: payee_accounts[0],
                            amount: requested_value,
                        });
                    }
                    _ => {
                        new_native_payments_ids.push(job_id);
                        ink_env::debug_println!(
                            "NATIVE PAYMENT with id: {} has failed",
                            current_job.id
                        );
                    }
                }
            }
            self.native_payments_ids = new_native_payments_ids;

            Ok(())
        }

        //For MANAGER ONLY
        #[ink(message)]
        // #[modifiers(only_role(ADMIN, MANAGER))]
        pub fn make_native_usd_payments(&mut self) -> Result<(), AccessControlError> {
            // self.native_payments_usd_ids.push(current_job.id);

            //GET ORACLE PRICE FOR DOT/USDT
            let price = self.get_average_price_for_pair(
                self.treasury_token_address,
                self.foreign_assets.get(&String::from("USDT")).unwrap(),
            );

            let mut new_native_usd_payments_ids = Vec::new();

            for job_id in self.native_payments_usd_ids.clone() {
                let current_job: JobInfo = self.jobs.get(&job_id).unwrap();
                let requested_value = current_job.requested_value; //this is USDT value in this case
                let amount = requested_value / price;

                let payee_accounts = current_job.payee_accounts;

                match self.make_transfer_to(self.treasury_token_address, payee_accounts[0], amount)
                {
                    Ok(()) => {
                        ink_env::debug_println!(
                            "NATIVE USD PAYMENT with id: {} and amount: {} has succeeded",
                            current_job.id,
                            amount
                        );

                        self.env().emit_event(ev_native_usd_payment {
                            job_id: job_id,
                            to: payee_accounts[0],
                            amount: amount,
                        });
                    }
                    _ => {
                        new_native_usd_payments_ids.push(job_id);
                        ink_env::debug_println!(
                            "NATIVE USD PAYMENT with id: {} has failed",
                            current_job.id
                        );
                    }
                }
            }
            self.native_payments_usd_ids = new_native_usd_payments_ids;

            Ok(())
        }

        //For MANAGER ONLY
        #[ink(message)]
        // #[modifiers(only_role(ADMIN, MANAGER))]
        pub fn make_non_native_payments(&mut self) -> Result<(), AccessControlError> {
            let mut new_non_native_payments_ids = Vec::new();

            for job_id in self.non_native_payments_ids.clone() {
                let current_job: JobInfo = self.jobs.get(&job_id).unwrap();

                //this is value in non native tokens (in this case we cover USDT)
                //and treasury tokens will be swapped for non_native tokens (USDT) and sent to the payee
                let requested_value = current_job.requested_value;
                //we asssume that in DEX all pairs are Token/USDT.

                //GET ORACLE PRICE FOR DOT/USDT
                let usdt_address = self.foreign_assets.get(&String::from("USDT")).unwrap();
                let price = self.get_price_for_pair(self.treasury_token_address, usdt_address);

                let amount = requested_value / price;
                let use_average_price = false;

                let payee_accounts = current_job.payee_accounts;

                self.execute_swap(
                    self.treasury_token_address,
                    usdt_address,
                    amount,
                    use_average_price,
                );

                match self.make_transfer_to(usdt_address, payee_accounts[0], requested_value) {
                    Ok(()) => {
                        ink_env::debug_println!(
                            "NON NATIVE PAYMENT with id: {} and requested_value in foreign asset: {} used treasury tokens amount: {} has succeeded",
                            current_job.id,
                            requested_value,
                            amount,
                        );

                        //EVENT
                        self.env().emit_event(ev_non_native_payment {
                            job_id: job_id,
                            to: payee_accounts[0],
                            amount: amount,
                        });
                    }
                    _ => {
                        new_non_native_payments_ids.push(job_id);
                        ink_env::debug_println!(
                            "NON NATIVE PAYMENT with id: {} has failed",
                            current_job.id
                        );
                    }
                }
            }
            self.non_native_payments_ids = new_non_native_payments_ids;

            Ok(())
        }

        #[ink(message)]
        #[modifiers(only_role(MANAGER))]
        pub fn set_liabilities_thresholds(
            &mut self,
            liabilities_threshold_level1: u8,
            // liabilities_threshold_level2: u8,
        ) -> Result<(), AccessControlError> {
            // assert!(
            //     liabilities_threshold_level1 < liabilities_threshold_level2,
            //     "liabilities_threshold_level1 must be lower than liabilities_threshold_level2"
            // );
            self.liabilities_thresholds = vec![
                (100 - liabilities_threshold_level1),
                (100 - liabilities_threshold_level1 * 2),
                (100 + liabilities_threshold_level1 * 3),
            ];
            Ok(())
        }

        #[ink(message)]
        #[modifiers(only_role(MANAGER))]
        pub fn set_check_points_intervals(
            &mut self,
            checkpoint1: u64,
            checkpoint2: u64,
            checkpoint3: u64,
        ) -> Result<(), AccessControlError> {
            self.check_points_intervals = vec![checkpoint1, checkpoint2, checkpoint3];
            Ok(())
        }

        //For MANAGER ONLY
        #[ink(message)]
        // #[modifiers(only_role(ADMIN, MANAGER))]
        pub fn calculate_liabilities(&mut self) -> Result<(), AccessControlError> {
            let treasury_tokens_balance = PSP22Ref::balance_of(
                &self.treasury_token_address,
                self.env().account_id().clone(),
            );

            ink_env::debug_println!(
                "calculate_liabilities treasury_tokens_balance: {}",
                treasury_tokens_balance
            );

            let current_timestamp = self.env().block_timestamp();
            // let current_timestamp = self.fake_timestamp;

            let timesstamp_2D = current_timestamp + self.check_points_intervals[0];
            let timesstamp_7D = current_timestamp + self.check_points_intervals[1];
            let timesstamp_30D = current_timestamp + self.check_points_intervals[2];

            ink_env::debug_println!(
                "calculate_liabilities current_timestamp: {} timesstamp_2D: {} timesstamp_7D: {} timesstamp_30D: {}",
                current_timestamp,timesstamp_2D,timesstamp_7D,timesstamp_30D
            );

            //Keeps hold of payments in treasury tokens e.g. DOT
            let mut liability_in_treasury: Balance = 0;
            let mut liability_in_treasury_2D: Balance = 0;
            let mut liability_in_treasury_7D: Balance = 0;
            let mut liability_in_treasury_30D: Balance = 0;

            //Keeps hold of payments in USDT
            let mut liability_in_usdt_tokens: Balance = 0;
            let mut liability_in_usdt_2D: Balance = 0;
            let mut liability_in_usdt_7D: Balance = 0;
            let mut liability_in_usdt_30D: Balance = 0;
            //Keeps hold of the above payments in USDT and the DOT equivalent
            let mut liability_in_usdt_tokens_treasury: Balance = 0;
            let mut liability_in_usdt_2D_treasury: Balance = 0;
            let mut liability_in_usdt_7D_treasury: Balance = 0;
            let mut liability_in_usdt_30D_treasury: Balance = 0;

            let usdt_address = self.foreign_assets.get(&String::from("USDT")).unwrap();
            let price = self.get_price_for_pair(self.treasury_token_address, usdt_address);

            ink_env::debug_println!(
                "calculate_liabilities price: {} usdt_address: {:?}",
                price,
                usdt_address
            );

            for job_id in self.open_jobs_ids.clone() {
                let current_job: JobInfo = self.jobs.get(&job_id).unwrap();

                if current_job.requested_token == self.treasury_token_address
                    && !current_job.value_in_usd
                {
                    for ts_pointer in current_job.next_installment_pointer
                        ..(current_job.payment_schedule.len() as u32)
                    {
                        if current_job.payment_schedule[ts_pointer as usize] <= timesstamp_2D {
                            liability_in_treasury_2D += current_job.requested_value;
                        }
                        if current_job.payment_schedule[ts_pointer as usize] <= timesstamp_7D {
                            liability_in_treasury_7D += current_job.requested_value;
                        }
                        if current_job.payment_schedule[ts_pointer as usize] <= timesstamp_30D {
                            liability_in_treasury_30D += current_job.requested_value;
                        }
                        liability_in_treasury += current_job.requested_value
                    }
                } else {
                    for ts_pointer in current_job.next_installment_pointer
                        ..(current_job.payment_schedule.len() as u32)
                    {
                        let amount_in_treasury_tokens = current_job.requested_value / price;
                        let usdt_value = current_job.requested_value;
                        if current_job.payment_schedule[ts_pointer as usize] <= timesstamp_2D {
                            liability_in_usdt_2D += usdt_value;
                            liability_in_usdt_2D_treasury += amount_in_treasury_tokens;
                        }
                        if current_job.payment_schedule[ts_pointer as usize] <= timesstamp_7D {
                            liability_in_usdt_7D += usdt_value;
                            liability_in_usdt_7D_treasury += amount_in_treasury_tokens;
                        }
                        if current_job.payment_schedule[ts_pointer as usize] <= timesstamp_30D {
                            liability_in_usdt_30D += usdt_value;
                            liability_in_usdt_30D_treasury += amount_in_treasury_tokens;
                        }
                        liability_in_usdt_tokens += usdt_value;
                        liability_in_usdt_tokens_treasury += amount_in_treasury_tokens;
                    }
                }
            }

            ink_env::debug_println!(
                "calculate_liabilities liability_in_treasury: {} liability_in_treasury_2D: {} liability_in_treasury_7D: {} liability_in_treasury_30D: {}",
                liability_in_treasury,liability_in_treasury_2D,liability_in_treasury_7D,liability_in_treasury_30D
            );
            ink_env::debug_println!(
                "calculate_liabilities liability_in_usdt_tokens: {} liability_in_usdt_2D: {} liability_in_usdt_7D: {} liability_in_usdt_30D: {}",
                liability_in_usdt_tokens,liability_in_usdt_2D,liability_in_usdt_7D,liability_in_usdt_30D
            );
            ink_env::debug_println!(
                "calculate_liabilities liability_in_usdt_tokens_treasury: {} liability_in_usdt_2D_treasury: {} liability_in_usdt_7D_treasury: {} liability_in_usdt_30D_treasury: {}",
                liability_in_usdt_tokens_treasury,liability_in_usdt_2D_treasury,liability_in_usdt_7D_treasury,liability_in_usdt_30D_treasury
            );

            for job_id in self.pending_jobs_ids.clone() {
                let current_job: JobInfo = self.jobs.get(&job_id).unwrap();

                if current_job.requested_token == self.treasury_token_address
                    && !current_job.value_in_usd
                {
                    for ts_pointer in current_job.next_installment_pointer
                        ..(current_job.payment_schedule.len() as u32)
                    {
                        if current_job.payment_schedule[ts_pointer as usize] <= timesstamp_2D {
                            liability_in_treasury_2D += current_job.requested_value;
                        }
                        if current_job.payment_schedule[ts_pointer as usize] <= timesstamp_7D {
                            liability_in_treasury_7D += current_job.requested_value;
                        }
                        if current_job.payment_schedule[ts_pointer as usize] <= timesstamp_30D {
                            liability_in_treasury_30D += current_job.requested_value;
                        }
                        liability_in_treasury += current_job.requested_value
                    }
                } else {
                    for ts_pointer in current_job.next_installment_pointer
                        ..(current_job.payment_schedule.len() as u32)
                    {
                        let amount_in_treasury_tokens = current_job.requested_value / price;
                        let usdt_value = current_job.requested_value;
                        if current_job.payment_schedule[ts_pointer as usize] <= timesstamp_2D {
                            liability_in_usdt_2D += usdt_value;
                            liability_in_usdt_2D_treasury += amount_in_treasury_tokens;
                        }
                        if current_job.payment_schedule[ts_pointer as usize] <= timesstamp_7D {
                            liability_in_usdt_7D += usdt_value;
                            liability_in_usdt_7D_treasury += amount_in_treasury_tokens;
                        }
                        if current_job.payment_schedule[ts_pointer as usize] <= timesstamp_30D {
                            liability_in_usdt_30D += usdt_value;
                            liability_in_usdt_30D_treasury += amount_in_treasury_tokens;
                        }
                        liability_in_usdt_tokens += usdt_value;
                        liability_in_usdt_tokens_treasury += amount_in_treasury_tokens;
                    }
                }
            }

            ink_env::debug_println!(
                "calculate_liabilities liability_in_treasury: {} liability_in_treasury_2D: {} liability_in_treasury_7D: {} liability_in_treasury_30D: {}",
                liability_in_treasury,liability_in_treasury_2D,liability_in_treasury_7D,liability_in_treasury_30D
            );
            ink_env::debug_println!(
                "calculate_liabilities liability_in_usdt_tokens: {} liability_in_usdt_2D: {} liability_in_usdt_7D: {} liability_in_usdt_30D: {}",
                liability_in_usdt_tokens,liability_in_usdt_2D,liability_in_usdt_7D,liability_in_usdt_30D
            );
            ink_env::debug_println!(
                "calculate_liabilities liability_in_usdt_tokens_treasury: {} liability_in_usdt_2D_treasury: {} liability_in_usdt_7D_treasury: {} liability_in_usdt_30D_treasury: {}",
                liability_in_usdt_tokens_treasury,liability_in_usdt_2D_treasury,liability_in_usdt_7D_treasury,liability_in_usdt_30D_treasury
            );

            let threshold_1 =
                (treasury_tokens_balance * (self.liabilities_thresholds[0 as usize]) as u128) / 100;
            let threshold_2 =
                (treasury_tokens_balance * (self.liabilities_thresholds[1 as usize]) as u128) / 100;

            let total_2D_treasury_liability =
                liability_in_treasury_2D + liability_in_usdt_2D_treasury;
            let mut total_2D_treasury_liability_state_health = 2;

            ink_env::debug_println!(
                "calculate_liabilities threshold_1: {} threshold_2: {} total_2D_treasury_liability: {} total_2D_treasury_liability_state_health: {} liability_in_treasury_2D {} liability_in_usdt_2D_treasury {}",
                threshold_1,threshold_2,total_2D_treasury_liability,total_2D_treasury_liability_state_health,liability_in_treasury_2D,liability_in_usdt_2D_treasury
            );

            if total_2D_treasury_liability > threshold_1 {
                let top_up_target = (total_2D_treasury_liability
                    * (self.liabilities_thresholds[2 as usize]) as u128)
                    / 100;
                let mut top_up_amount = 0;
                if top_up_target >= treasury_tokens_balance {
                    top_up_amount = top_up_target - treasury_tokens_balance
                }

                ink_env::debug_println!(
                    "liability_in_treasury_2D: {} is above liabilities_thresholds LEVEL 1: {} treasury_tokens_balance: {} top_up_target: {} TOP UP NOW WITH  top_up_amount: {}",
                    total_2D_treasury_liability,
                    threshold_1, treasury_tokens_balance, top_up_target,  top_up_amount
                );
                total_2D_treasury_liability_state_health = 0;
                //EMIT EVENT
                self.env().emit_event(liability_threshold_breached_top {
                    level_type: 2,
                    current_balance: treasury_tokens_balance,
                    top_up_amount: top_up_amount,
                });
            } else if total_2D_treasury_liability > threshold_2 {
                ink_env::debug_println!(
                    "liability_in_treasury_2D: {} is above liabilities_thresholds LEVEL 2: {} CONSIDER TOPPING UP",
                    total_2D_treasury_liability,
                    threshold_2
                );
                total_2D_treasury_liability_state_health = 1;
                //EMIT EVENT
                self.env().emit_event(liability_threshold_breached_med {
                    level_type: 2,
                    current_balance: treasury_tokens_balance,
                    liability: total_2D_treasury_liability,
                });
            } else {
                ink_env::debug_println!(
                    "liability_in_treasury_2D: {} ALL GOOD is above liabilities_thresholds LEVEL 2: {}",
                    total_2D_treasury_liability,
                    threshold_2
                );
                total_2D_treasury_liability_state_health = 2;
            }

            let total_7D_treasury_liability =
                liability_in_treasury_7D + liability_in_usdt_7D_treasury;
            let mut total_7D_treasury_liability_state_health = 2;
            if total_7D_treasury_liability > threshold_1 {
                let top_up_target = (total_7D_treasury_liability
                    * (self.liabilities_thresholds[2 as usize]) as u128)
                    / 100;
                let mut top_up_amount = 0;
                if top_up_target >= treasury_tokens_balance {
                    top_up_amount = top_up_target - treasury_tokens_balance
                }

                ink_env::debug_println!(
                    "total_7D_treasury_liability: {} is above liabilities_thresholds LEVEL 1: {} treasury_tokens_balance: {} top_up_target: {} TOP UP NOW WITH  top_up_amount: {}",
                    total_7D_treasury_liability,
                    threshold_1, treasury_tokens_balance, top_up_target,  top_up_amount
                );
                total_7D_treasury_liability_state_health = 0;
                //EMIT EVENT
                self.env().emit_event(liability_threshold_breached_top {
                    level_type: 7,
                    current_balance: treasury_tokens_balance,
                    top_up_amount: top_up_amount,
                });
            } else if total_7D_treasury_liability > threshold_2 {
                ink_env::debug_println!(
                    "total_7D_treasury_liability: {} is above liabilities_thresholds LEVEL 2: {} CONSIDER TOPPING UP",
                    total_7D_treasury_liability,
                    threshold_2
                );
                total_7D_treasury_liability_state_health = 1;
                //EMIT EVENT
                self.env().emit_event(liability_threshold_breached_med {
                    level_type: 7,
                    current_balance: treasury_tokens_balance,
                    liability: total_7D_treasury_liability,
                });
            } else {
                ink_env::debug_println!(
                    "total_7D_treasury_liability: {} ALL GOOD is above liabilities_thresholds LEVEL 2: {}",
                    total_7D_treasury_liability,
                    threshold_2
                );
                total_7D_treasury_liability_state_health = 2;
            }

            let total_30D_treasury_liability =
                liability_in_treasury_30D + liability_in_usdt_30D_treasury;
            let mut total_30D_treasury_liability_state_health = 2;
            if total_30D_treasury_liability > threshold_1 {
                let top_up_target = (total_30D_treasury_liability
                    * (self.liabilities_thresholds[2 as usize]) as u128)
                    / 100;
                let mut top_up_amount = 0;
                if top_up_target >= treasury_tokens_balance {
                    top_up_amount = top_up_target - treasury_tokens_balance
                }

                ink_env::debug_println!(
                    "total_30D_treasury_liability: {} is above liabilities_thresholds LEVEL 1: {} treasury_tokens_balance: {} top_up_target: {} TOP UP NOW WITH  top_up_amount: {}",
                    total_30D_treasury_liability,
                    threshold_1, treasury_tokens_balance, top_up_target,  top_up_amount
                );
                total_30D_treasury_liability_state_health = 0;
                //EMIT EVENT
                self.env().emit_event(liability_threshold_breached_top {
                    level_type: 30,
                    current_balance: treasury_tokens_balance,
                    top_up_amount: top_up_amount,
                });
            } else if total_30D_treasury_liability > threshold_2 {
                ink_env::debug_println!(
                    "total_30D_treasury_liability: {} is above liabilities_thresholds LEVEL 2: {} CONSIDER TOPPING UP",
                    total_30D_treasury_liability,
                    threshold_2
                );
                total_30D_treasury_liability_state_health = 1;
                //EMIT EVENT
                self.env().emit_event(liability_threshold_breached_med {
                    level_type: 30,
                    current_balance: treasury_tokens_balance,
                    liability: total_30D_treasury_liability,
                });
            } else {
                ink_env::debug_println!(
                    "total_30D_treasury_liability: {} ALL GOOD is above liabilities_thresholds LEVEL 2: {}",
                    total_30D_treasury_liability,
                    threshold_2
                );
                total_30D_treasury_liability_state_health = 2;
            }

            let total_treasury_liability =
                liability_in_treasury + liability_in_usdt_tokens_treasury;
            let mut total_treasury_liability_state_health = 2;
            if total_treasury_liability > threshold_1 {
                let top_up_target = (total_treasury_liability
                    * (self.liabilities_thresholds[2 as usize]) as u128)
                    / 100;
                let mut top_up_amount = 0;
                if top_up_target >= treasury_tokens_balance {
                    top_up_amount = top_up_target - treasury_tokens_balance
                }

                ink_env::debug_println!(
                    "total_treasury_liability: {} is above liabilities_thresholds LEVEL 1: {} treasury_tokens_balance: {} top_up_target: {} TOP UP NOW WITH  top_up_amount: {}",
                    total_treasury_liability,
                    threshold_1, treasury_tokens_balance, top_up_target,  top_up_amount
                );
                total_treasury_liability_state_health = 0;
                //EMIT EVENT
                self.env().emit_event(liability_threshold_breached_top {
                    level_type: 0,
                    current_balance: treasury_tokens_balance,
                    top_up_amount: top_up_amount,
                });
            } else if total_treasury_liability > threshold_2 {
                ink_env::debug_println!(
                    "total_treasury_liability: {} is above liabilities_thresholds LEVEL 2: {}  CONSIDER TOPPING UP",
                    total_treasury_liability,
                    threshold_2
                );
                total_treasury_liability_state_health = 1;
                //EMIT EVENT
                self.env().emit_event(liability_threshold_breached_med {
                    level_type: 0,
                    current_balance: treasury_tokens_balance,
                    liability: total_treasury_liability,
                });
            } else {
                ink_env::debug_println!(
                    "total_treasury_liability: {} ALL GOOD is above liabilities_thresholds LEVEL 2: {}",
                    total_treasury_liability,
                    threshold_2
                );
                total_treasury_liability_state_health = 2;
            }

            self.liability_in_treasury = vec![
                liability_in_treasury,
                liability_in_treasury_2D,
                liability_in_treasury_7D,
                liability_in_treasury_30D,
            ];
            self.liability_in_usdt_tokens = vec![
                liability_in_usdt_tokens,
                liability_in_usdt_2D,
                liability_in_usdt_7D,
                liability_in_usdt_30D,
            ];
            self.liability_in_usdt_tokens_treasury = vec![
                liability_in_usdt_tokens_treasury,
                liability_in_usdt_2D_treasury,
                liability_in_usdt_7D_treasury,
                liability_in_usdt_30D_treasury,
            ];
            self.liability_health = vec![
                total_treasury_liability_state_health,
                total_2D_treasury_liability_state_health,
                total_7D_treasury_liability_state_health,
                total_30D_treasury_liability_state_health,
            ];

            Ok(())
        }

        //For MANAGER ONLY
        // #[ink(message)]
        // #[modifiers(only_role(ADMIN, MANAGER))]
        pub fn make_transfer_to(
            &mut self,
            token_address: AccountId,
            to: AccountId,
            amount: Balance,
        ) -> Result<(), AccessControlError> {
            PSP22Ref::transfer(&token_address, to, amount, Vec::<u8>::new())
                .expect("Transfer to external account did not go well");

            //SHOULD EMMIT EVENT

            Ok(())
        }

        #[ink(message)]
        #[modifiers(only_role(MANAGER))]
        pub fn register_foreign_asset(
            &mut self,
            token_symbol: String,
            token_address: AccountId,
        ) -> Result<(), AccessControlError> {
            match self.foreign_assets.get(&token_symbol) {
                Some(_) => (),
                None => {
                    self.foreign_assets.insert(&token_symbol, &token_address);
                    self.foreign_assets_vec.push(token_symbol);
                }
            }

            Ok(())
        }

        // ***        ***
        // *** ORACLE ***

        ///Set the Oracle_DEX Address that will be used
        #[ink(message)]
        #[modifiers(only_role(MANAGER))]
        pub fn set_oracle_dex_address(
            &mut self,
            oracle_dex_address: AccountId,
        ) -> Result<(), AccessControlError> {
            self.oracle_dex_address = oracle_dex_address;
            Ok(())
        }

        ///Get Pair Price from Oracle
        #[ink(message)]
        pub fn get_price_for_pair(
            &self,
            // contract_address: AccountId,
            base_token: AccountId,
            quote_token: AccountId,
        ) -> Balance {
            // OracleDexRef::get_pair_price(&contract_address, base_token, quote_token)
            OracleDexRef::get_pair_price(&self.oracle_dex_address, base_token, quote_token)
        }

        ///Get Average Pair Price from Oracle
        #[ink(message)]
        pub fn get_average_price_for_pair(
            &self,
            // contract_address: AccountId,
            base_token: AccountId,
            quote_token: AccountId,
        ) -> Balance {
            OracleDexRef::get_average_price(&self.oracle_dex_address, base_token, quote_token)
        }

        #[ink(message)]
        pub fn execute_swap(
            &mut self,
            // contract_address: AccountId,
            deposited_token: AccountId,
            withdrawn_token: AccountId,
            amount: Balance,
            use_average_price: bool,
        ) {
            let contract_address = self.oracle_dex_address;

            PSP22Ref::approve(&deposited_token, contract_address, amount)
                .expect("Approval for deposited_token did not go as planned");

            let withdrawn_amount = OracleDexRef::swap(
                &contract_address,
                deposited_token,
                withdrawn_token,
                amount,
                self.env().caller().clone(),
                use_average_price,
            );

            match withdrawn_amount {
                Ok(w_amount) => {
                    if w_amount > 0 {
                        PSP22Ref::transfer_from_builder(
                            &withdrawn_token,
                            contract_address,
                            self.env().account_id().clone(),
                            w_amount,
                            Vec::<u8>::new(),
                        )
                        .call_flags(ink_env::CallFlags::default().set_allow_reentry(true))
                        .fire()
                        .unwrap();
                    }
                }
                _ => (),
            }
        }
        // *** ORACLE ***
        // ***        ***
    }
}
