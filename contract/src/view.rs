use crate::*;


#[near_bindgen]
impl Contract {
    /// Show the current exchange price
    pub fn exchange_price(&self) -> U128 {
        self.exchange_price_in_yocto_near
    }

    pub fn subscriptions_of(&self, account_id: AccountId) -> Vec<subscriptions::UserSubscription> {
        if let Some(subscriptions) = self.subscriptions.get(&account_id) {
            // TODO: add pagination
            return subscriptions
        }
        vec![]
    }
}
