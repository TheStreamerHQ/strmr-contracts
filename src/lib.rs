/*!
Fungible Token implementation with JSON serialization.
NOTES:
  - The maximum balance value is limited by U128 (2**128 - 1).
  - JSON calls should pass U128 as a base-10 string. E.g. "100".
  - The contract optimizes the inner trie structure by hashing account IDs. It will prevent some
    abuse of deep tries. Shouldn't be an issue, once NEAR clients implement full hashing of keys.
  - The contract tracks the change in storage before and after the call. If the storage increases,
    the contract requires the caller of the contract to attach enough deposit to the function call
    to cover the storage cost.
    This is done to prevent a denial of service attack on the contract by taking all available storage.
    If the storage decreases, the contract will issue a refund for the cost of the released storage.
    The unused tokens from the attached deposit are also refunded, so it's safe to
    attach more deposit than required.
  - To prevent the deployed contract from being modified or deleted, it should not have any access
    keys on its account.
*/
use near_contract_standards::fungible_token::metadata::{
    FungibleTokenMetadata, FungibleTokenMetadataProvider, FT_METADATA_SPEC,
};
use near_contract_standards::fungible_token::FungibleToken;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::LazyOption;
use near_sdk::json_types::{ValidAccountId, U128};
use near_sdk::{env, log, near_bindgen, ext_contract, AccountId, Balance, PanicOnDefault, PromiseOrValue, Promise};

mod owner;
mod public;
mod view;

near_sdk::setup_alloc!();

#[ext_contract(ext_owner_methods)]
pub trait ExtOwnerMethods {
    fn print_tokens(&mut self, amount: U128);

    fn replace_exchange_price(&mut self, new_price_in_yocto_nears: U128);

    fn charge_users(&mut self, charge_list: Vec<(ValidAccountId, Balance)>);
}

#[ext_contract(ext_view_methods)]
pub trait ExtViewMethods {
    pub fn exchange_price(&self) -> U128;
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Contract {
    exchange_price_in_yocto_near: U128,
    owner_id: AccountId,
    token: FungibleToken,
    metadata: LazyOption<FungibleTokenMetadata>,
}

const DATA_IMAGE_SVG_NEAR_ICON: &str = "data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 288 288'%3E%3Cg id='l' data-name='l'%3E%3Cpath d='M187.58,79.81l-30.1,44.69a3.2,3.2,0,0,0,4.75,4.2L191.86,103a1.2,1.2,0,0,1,2,.91v80.46a1.2,1.2,0,0,1-2.12.77L102.18,77.93A15.35,15.35,0,0,0,90.47,72.5H87.34A15.34,15.34,0,0,0,72,87.84V201.16A15.34,15.34,0,0,0,87.34,216.5h0a15.35,15.35,0,0,0,13.08-7.31l30.1-44.69a3.2,3.2,0,0,0-4.75-4.2L96.14,186a1.2,1.2,0,0,1-2-.91V104.61a1.2,1.2,0,0,1,2.12-.77l89.55,107.23a15.35,15.35,0,0,0,11.71,5.43h3.13A15.34,15.34,0,0,0,216,201.16V87.84A15.34,15.34,0,0,0,200.66,72.5h0A15.35,15.35,0,0,0,187.58,79.81Z'/%3E%3C/g%3E%3C/svg%3E";

#[near_bindgen]
impl Contract {
    /// Initializes the contract with the given total supply owned by the given `owner_id` with
    /// the given fungible token metadata.
    #[init]
    pub fn new(
        owner_id: ValidAccountId,
        total_supply: U128,
        exchange_price_in_yocto_near: U128,
        metadata: FungibleTokenMetadata,
    ) -> Self {
        assert!(!env::state_exists(), "Already initialized");
        metadata.assert_valid();
        let mut this = Self {
            owner_id: owner_id.as_ref().into(),
            exchange_price_in_yocto_near,
            token: FungibleToken::new(b"a".to_vec()),
            metadata: LazyOption::new(b"m".to_vec(), Some(&metadata)),
        };
        this.token.internal_register_account(owner_id.as_ref());
        this.token.internal_deposit(owner_id.as_ref(), total_supply.into());
        this
    }

    fn on_account_closed(&mut self, account_id: AccountId, balance: Balance) {
        log!("Closed @{} with {}", account_id, balance);
    }

    fn on_tokens_burned(&mut self, account_id: AccountId, amount: Balance) {
        log!("Account @{} burned {}", account_id, amount);
    }
}

near_contract_standards::impl_fungible_token_core!(Contract, token, on_tokens_burned);
near_contract_standards::impl_fungible_token_storage!(Contract, token, on_account_closed);

#[near_bindgen]
impl FungibleTokenMetadataProvider for Contract {
    fn ft_metadata(&self) -> FungibleTokenMetadata {
        self.metadata.get().unwrap()
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use near_sdk::test_utils::{accounts, VMContextBuilder};
    use near_sdk::MockedBlockchain;
    use near_sdk::{testing_env, Balance};

    use super::*;

    const TOTAL_SUPPLY: Balance = 1_000_000_000_000_000;
    const EXCHANGE_PRICE: u128 = 1_000_000_000_000_000_000_000_000;

    fn get_context(predecessor_account_id: ValidAccountId) -> VMContextBuilder {
        let mut builder = VMContextBuilder::new();
        builder
            .current_account_id(accounts(0))
            .signer_account_id(predecessor_account_id.clone())
            .predecessor_account_id(predecessor_account_id);
        builder
    }

    #[test]
    fn test_new() {
        let mut context = get_context(accounts(1));
        testing_env!(context.build());
        let contract = Contract::new(
            accounts(1).into(),
            TOTAL_SUPPLY.into(),
            EXCHANGE_PRICE.into(),
            FungibleTokenMetadata {
                spec: FT_METADATA_SPEC.to_string(),
                name: "Example NEAR fungible token".to_string(),
                symbol: "EXAMPLE".to_string(),
                icon: Some(DATA_IMAGE_SVG_NEAR_ICON.to_string()),
                reference: None,
                reference_hash: None,
                decimals: 0,
            });
        testing_env!(context.is_view(true).build());
        assert_eq!(contract.ft_total_supply().0, TOTAL_SUPPLY);
        assert_eq!(contract.ft_balance_of(accounts(1)).0, TOTAL_SUPPLY);
    }

    #[test]
    #[should_panic(expected = "The contract is not initialized")]
    fn test_default() {
        let context = get_context(accounts(1));
        testing_env!(context.build());
        let _contract = Contract::default();
    }

    #[test]
    fn test_transfer() {
        let mut context = get_context(accounts(2));
        testing_env!(context.build());
        let mut contract = Contract::new(
            accounts(2).into(),
            TOTAL_SUPPLY.into(),
            EXCHANGE_PRICE.into(),
            FungibleTokenMetadata {
                spec: FT_METADATA_SPEC.to_string(),
                name: "Example NEAR fungible token".to_string(),
                symbol: "EXAMPLE".to_string(),
                icon: Some(DATA_IMAGE_SVG_NEAR_ICON.to_string()),
                reference: None,
                reference_hash: None,
                decimals: 0,
            });
        testing_env!(context
            .storage_usage(env::storage_usage())
            .attached_deposit(contract.storage_balance_bounds().min.into())
            .predecessor_account_id(accounts(1))
            .build());
        // Paying for account registration, aka storage deposit
        contract.storage_deposit(None, None);

        testing_env!(context
            .storage_usage(env::storage_usage())
            .attached_deposit(1)
            .predecessor_account_id(accounts(2))
            .build());
        let transfer_amount = TOTAL_SUPPLY / 3;
        contract.ft_transfer(accounts(1), transfer_amount.into(), None);

        testing_env!(context
            .storage_usage(env::storage_usage())
            .account_balance(env::account_balance())
            .is_view(true)
            .attached_deposit(0)
            .build());
        assert_eq!(contract.ft_balance_of(accounts(2)).0, (TOTAL_SUPPLY - transfer_amount));
        assert_eq!(contract.ft_balance_of(accounts(1)).0, transfer_amount);
    }

    #[test]
    fn test_print_tokens() {
        let tokens_to_print: Balance = 1_000;
        let context = get_context(accounts(1));
        testing_env!(context.build());
        let mut contract = Contract::new(
            accounts(1).into(),
            TOTAL_SUPPLY.into(),
            EXCHANGE_PRICE.into(),
            FungibleTokenMetadata {
                spec: FT_METADATA_SPEC.to_string(),
                name: "Example NEAR fungible token".to_string(),
                symbol: "EXAMPLE".to_string(),
                icon: Some(DATA_IMAGE_SVG_NEAR_ICON.to_string()),
                reference: None,
                reference_hash: None,
                decimals: 0,
            });

        assert_eq!(contract.ft_total_supply().0, TOTAL_SUPPLY);
        assert_eq!(contract.ft_balance_of(accounts(1)).0, TOTAL_SUPPLY);

        contract.print_tokens(tokens_to_print.into());
        assert_eq!(contract.ft_total_supply().0, (TOTAL_SUPPLY + tokens_to_print));
    }

    #[test]
    fn test_print_tokens_by_non_owner_must_fail() {
        let tokens_to_print: Balance = 1_000;
        let context = get_context(accounts(2));
        testing_env!(context.build());
        let mut contract = Contract::new(
            accounts(1).into(),
            TOTAL_SUPPLY.into(),
            EXCHANGE_PRICE.into(),
            FungibleTokenMetadata {
                spec: FT_METADATA_SPEC.to_string(),
                name: "Example NEAR fungible token".to_string(),
                symbol: "EXAMPLE".to_string(),
                icon: Some(DATA_IMAGE_SVG_NEAR_ICON.to_string()),
                reference: None,
                reference_hash: None,
                decimals: 0,
            });

        assert_eq!(contract.ft_total_supply().0, TOTAL_SUPPLY);
        assert_eq!(contract.ft_balance_of(accounts(1)).0, TOTAL_SUPPLY);

        let result = std::panic::catch_unwind(move || contract.print_tokens(tokens_to_print.into()));
        assert!(result.is_err());
    }

    #[test]
    fn test_replace_price() {
        let new_price_in_yocto_nears: u128 = 10_000_000_000_000_000_000_000_000;
        let context = get_context(accounts(1));
        testing_env!(context.build());
        let mut contract = Contract::new(
            accounts(1).into(),
            TOTAL_SUPPLY.into(),
            EXCHANGE_PRICE.into(),
            FungibleTokenMetadata {
                spec: FT_METADATA_SPEC.to_string(),
                name: "Example NEAR fungible token".to_string(),
                symbol: "EXAMPLE".to_string(),
                icon: Some(DATA_IMAGE_SVG_NEAR_ICON.to_string()),
                reference: None,
                reference_hash: None,
                decimals: 0,
            });

        assert_eq!(contract.exchange_price().0, EXCHANGE_PRICE);

        contract.replace_exchange_price(new_price_in_yocto_nears.into());
        assert_eq!(contract.exchange_price().0, new_price_in_yocto_nears);
    }

    #[test]
    fn test_replace_price_by_non_owner_must_fail() {
        let new_price: u128 = 1_000;
        let context = get_context(accounts(2));
        testing_env!(context.build());
        let mut contract = Contract::new(
            accounts(1).into(),
            TOTAL_SUPPLY.into(),
            EXCHANGE_PRICE.into(),
            FungibleTokenMetadata {
                spec: FT_METADATA_SPEC.to_string(),
                name: "Example NEAR fungible token".to_string(),
                symbol: "EXAMPLE".to_string(),
                icon: Some(DATA_IMAGE_SVG_NEAR_ICON.to_string()),
                reference: None,
                reference_hash: None,
                decimals: 0,
            });

        assert_eq!(contract.exchange_price().0, EXCHANGE_PRICE);

        let result = std::panic::catch_unwind(move || contract.replace_exchange_price(new_price.into()));
        assert!(result.is_err());
    }

    #[test]
    fn test_buy_ft_tokens() {
        // We want to buy 10 ft_tokens and get half the NEAR refunded
        let deposit_to_attach: u128 = EXCHANGE_PRICE * 10 + EXCHANGE_PRICE / 2;
        let mut context = get_context(accounts(2));
        testing_env!(context.build());
        let mut contract = Contract::new(
            accounts(1).into(),
            TOTAL_SUPPLY.into(),
            EXCHANGE_PRICE.into(),
            FungibleTokenMetadata {
                spec: FT_METADATA_SPEC.to_string(),
                name: "Example NEAR fungible token".to_string(),
                symbol: "EXAMPLE".to_string(),
                icon: Some(DATA_IMAGE_SVG_NEAR_ICON.to_string()),
                reference: None,
                reference_hash: None,
                decimals: 0,
            });

        testing_env!(context
            .storage_usage(env::storage_usage())
            .attached_deposit(contract.storage_balance_bounds().min.into())
            .predecessor_account_id(accounts(2))
            .build());
        // Paying for account registration, aka storage deposit
        contract.storage_deposit(None, None);

        testing_env!(context
            .predecessor_account_id(accounts(2))
            .attached_deposit(deposit_to_attach)
            .build()
        );
        contract.buy_ft_tokens();

        assert_eq!(contract.ft_balance_of(accounts(2)).0, 10);
    }

    #[test]
    fn test_charge_users() {
        // We want to buy 10 ft_tokens
        let deposit_to_attach: u128 = EXCHANGE_PRICE * 10;
        let mut context = get_context(accounts(2));
        testing_env!(context.build());
        let mut contract = Contract::new(
            accounts(1).into(),
            TOTAL_SUPPLY.into(),
            EXCHANGE_PRICE.into(),
            FungibleTokenMetadata {
                spec: FT_METADATA_SPEC.to_string(),
                name: "Example NEAR fungible token".to_string(),
                symbol: "EXAMPLE".to_string(),
                icon: Some(DATA_IMAGE_SVG_NEAR_ICON.to_string()),
                reference: None,
                reference_hash: None,
                decimals: 0,
            });

        // User buys 10 FT tokens
        testing_env!(context
            .storage_usage(env::storage_usage())
            .attached_deposit(contract.storage_balance_bounds().min.into())
            .predecessor_account_id(accounts(2))
            .build());
        // Paying for account registration, aka storage deposit
        contract.storage_deposit(None, None);

        testing_env!(context
            .predecessor_account_id(accounts(2))
            .attached_deposit(deposit_to_attach)
            .build()
        );
        contract.buy_ft_tokens();

        assert_eq!(contract.ft_balance_of(accounts(2)).0, 10);

        testing_env!(context
            .signer_account_id(accounts(1))
            .attached_deposit(0)
            .build()
        );

        // charge user for 5
        contract.charge_users(vec![(accounts(2), 5u128)]);

        assert_eq!(contract.ft_balance_of(accounts(2)).0, 5);
    }

    #[test]
    fn test_charge_users_by_non_owner_must_fail() {
        let context = get_context(accounts(1));
        testing_env!(context.build());
        let mut contract = Contract::new(
            accounts(1).into(),
            TOTAL_SUPPLY.into(),
            EXCHANGE_PRICE.into(),
            FungibleTokenMetadata {
                spec: FT_METADATA_SPEC.to_string(),
                name: "Example NEAR fungible token".to_string(),
                symbol: "EXAMPLE".to_string(),
                icon: Some(DATA_IMAGE_SVG_NEAR_ICON.to_string()),
                reference: None,
                reference_hash: None,
                decimals: 0,
            });

        let result = std::panic::catch_unwind(move || contract.charge_users(vec![(accounts(2), 5u128)]));
        assert!(result.is_err());
    }
}
