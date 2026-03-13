use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SavedItemPrice {
    pub item_name: String,
    pub price_input: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SavedCraft {
    pub name: String,
    pub recipe_text: String,
    pub sell_price_input: String,
    #[serde(default)]
    pub item_prices: Vec<SavedItemPrice>,
}
