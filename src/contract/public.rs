use super::*;

// * PUBLIC methods *
#[near_bindgen]
impl Contract {
    /// Exchange NEAR tokens for FT tokens based on current exchange price
    #[payable]
    pub fn buy_ft_tokens(&mut self) {
        let attached_deposit = env::attached_deposit();
        let signer_account_id = env::signer_account_id();
        // Calculate how many ft_tokens signer can get in exchange for the attached_deposit
        let affordable_amount: u128 = attached_deposit / self.exchange_price_in_yocto_near.0;

        // Calculate surplus that should be refunded
        let surplus: u128 = attached_deposit - (affordable_amount * self.exchange_price_in_yocto_near.0);
        // Transfer bought ft_tokens to the signer
        self.token.internal_transfer(&self.owner_id, &signer_account_id, affordable_amount, None);

        // Send spent yoctoNEARs to the treasury (self.owner_id)
        Promise::new(self.owner_id.clone()).transfer(attached_deposit - surplus);
        // Refund surplus yoctoNEARs to the signer
        Promise::new(signer_account_id.clone()).transfer(surplus);
        log!(
            "Account @{} has bought {} ${} tokens. Refunded {} yoctoNEARS",
            signer_account_id,
            affordable_amount,
            &self.metadata.get().unwrap().symbol,
            surplus,
        );
    }
}
