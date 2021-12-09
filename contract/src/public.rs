use crate::*;

#[near_bindgen]
impl Contract {
    /// Exchange NEAR tokens for FT tokens based on current exchange price
    #[payable]
    pub fn buy_ft_tokens(&mut self) {
        let attached_deposit = env::attached_deposit();
        let signer_account_id = env::signer_account_id();

        if self.token.accounts.get(&signer_account_id).is_none() {
            self.token.internal_register_account(&signer_account_id);
        }

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

    pub fn create_subscription(
        &mut self,
        endpoint: String,
        event: subscriptions::Event,
    ) {
        let signer_account_id = env::signer_account_id();
        let mut subscription_list = self.get_or_create_user_subscription_list(&signer_account_id);
        let subscription_id = subscription_list
                .iter()
                .map(|subscription| subscription.id)
                .max()
                .unwrap_or(0) + 1;
        let new_user_subscription = subscriptions::UserSubscription {
            id: subscription_id,
            enabled: true,
            endpoint,
            event,
        };
        subscription_list.push(new_user_subscription);
        self.subscriptions.insert(&signer_account_id, &subscription_list);
    }

    pub fn delete_subscription(
        &mut self,
        id: u8,
    ) {
        let signer_account_id = env::signer_account_id();
        let mut subscription_list = self.get_or_create_user_subscription_list(&signer_account_id);
        let index_of_subscription_to_delete = subscription_list
            .iter()
            .position(|subscription| subscription.id == id);
        if let Some(index) = index_of_subscription_to_delete {
            subscription_list.remove(index);
            self.subscriptions.insert(&signer_account_id, &subscription_list);
        } else {
            panic!(
                "Account {} doesn't have the Subscription with id {}",
                &signer_account_id,
                id,
            );
        }
    }
}


impl Contract {
    fn get_or_create_user_subscription_list(&mut self, account_id: &AccountId) -> Vec<subscriptions::UserSubscription> {
        match self.subscriptions.get(account_id) {
            Some(subscription_list) => subscription_list,
            None => {
                let subscription_list: Vec<subscriptions::UserSubscription> = Vec::new();
                self.subscriptions.insert(account_id, &subscription_list);
                subscription_list
            }
        }
    }
}
