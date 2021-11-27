use super::*;

// * OWNER methods *
#[near_bindgen]
impl Contract {
    /// Create additional provided amount of FT tokens in circulation
    pub fn print_tokens(&mut self, amount: U128) {
        assert_eq!(self.owner_id, env::signer_account_id(), "Signer must be an owner");
        let tokens_to_print: u128 = amount.into();
        self.token.internal_deposit(&self.owner_id, tokens_to_print);
        log!("{} tokens were printed and deposited to owner's account", tokens_to_print);
    }

    /// Set a new exchange price for FT token
    pub fn replace_exchange_price(&mut self, new_price_in_yocto_nears: U128) {
        assert_eq!(self.owner_id, env::signer_account_id(), "Signer must be an owner");
        self.exchange_price_in_yocto_near = new_price_in_yocto_nears;
        log!("Exchange price has been changed to the new value (in yoctoNEARS) of {:?}", new_price_in_yocto_nears)
    }

    /// Charge specified users for a specified amount of FT tokens
    pub fn charge_users(&mut self, charge_list: Vec<(ValidAccountId, Balance)>) {
        assert_eq!(self.owner_id, env::signer_account_id(), "Signer must be an owner");
        for (valid_account_id, balance_to_burn) in charge_list.iter() {
            let account_id: String = valid_account_id.clone().into();
            let account_available_balance = self.token.accounts.get(&account_id).unwrap_or(0);
            if account_available_balance >= *balance_to_burn {
                self.token.internal_withdraw(&account_id, *balance_to_burn);
                log!(
                    "Account @{} charged for {} ${}",
                    account_id,
                    balance_to_burn,
                    &self.metadata.get().unwrap().symbol,
                );
            } else {
                self.token.internal_withdraw(&account_id, account_available_balance);
                log!(
                    "Account @{} charged for entire balance ({}). Supposed to charge {} ${}",
                    account_id,
                    account_available_balance,
                    balance_to_burn,
                    &self.metadata.get().unwrap().symbol,
                );
            }
        }
    }
}
