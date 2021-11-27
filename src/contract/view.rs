use super::*;

// * VIEW methods *
#[near_bindgen]
impl Contract {
    /// Show the current exchange price
    pub fn exchange_price(&self) -> U128 {
        self.exchange_price_in_yocto_near
    }
}
