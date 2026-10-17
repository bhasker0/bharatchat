use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CatalogError {
    #[error("Product item not found: {0}")]
    NotFound(String),

    #[error("Invalid product price: must be greater than zero")]
    InvalidPrice,

    #[error("Catalog validation error: {0}")]
    Validation(String),
}

pub type Result<T> = std::result::Result<T, CatalogError>;

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
pub enum ItemAvailability {
    InStock,
    OutOfStock,
    PreOrder,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct ProductItem {
    pub id: String,
    pub retailer_id: String,
    pub name: String,
    pub description: String,
    pub price_inr: f64,
    pub currency: String,
    pub image_cids: Vec<String>, // BLAKE3 CIDs hosted in sovereign-storage
    pub category: String,
    pub availability: ItemAvailability,
    pub sku: String,
}

impl ProductItem {
    pub fn new(
        id: &str,
        retailer_id: &str,
        name: &str,
        description: &str,
        price_inr: f64,
        category: &str,
        sku: &str,
    ) -> Result<Self> {
        if price_inr <= 0.0 {
            return Err(CatalogError::InvalidPrice);
        }

        Ok(Self {
            id: id.to_string(),
            retailer_id: retailer_id.to_string(),
            name: name.to_string(),
            description: description.to_string(),
            price_inr,
            currency: "INR".to_string(),
            image_cids: Vec::new(),
            category: category.to_string(),
            availability: ItemAvailability::InStock,
            sku: sku.to_string(),
        })
    }

    pub fn add_image_cid(&mut self, cid: &str) {
        self.image_cids.push(cid.to_string());
    }

    /// Converts product to standard schema.org JSON-LD object for interoperability
    pub fn to_jsonld(&self) -> serde_json::Value {
        serde_json::json!({
            "@context": "https://schema.org/",
            "@type": "Product",
            "identifier": self.id,
            "sku": self.sku,
            "name": self.name,
            "description": self.description,
            "category": self.category,
            "offers": {
                "@type": "Offer",
                "priceCurrency": self.currency,
                "price": format!("{:.2}", self.price_inr),
                "availability": match self.availability {
                    ItemAvailability::InStock => "https://schema.org/InStock",
                    ItemAvailability::OutOfStock => "https://schema.org/OutOfStock",
                    ItemAvailability::PreOrder => "https://schema.org/PreOrder",
                }
            },
            "image": self.image_cids
        })
    }
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct ProductCatalog {
    pub catalog_id: String,
    pub merchant_did: String,
    pub catalog_name: String,
    pub items: Vec<ProductItem>,
}

impl ProductCatalog {
    pub fn new(catalog_id: &str, merchant_did: &str, catalog_name: &str) -> Self {
        Self {
            catalog_id: catalog_id.to_string(),
            merchant_did: merchant_did.to_string(),
            catalog_name: catalog_name.to_string(),
            items: Vec::new(),
        }
    }

    pub fn add_item(&mut self, item: ProductItem) {
        self.items.push(item);
    }

    pub fn find_item(&self, item_id: &str) -> Result<&ProductItem> {
        self.items
            .iter()
            .find(|i| i.id == item_id || i.retailer_id == item_id)
            .ok_or_else(|| CatalogError::NotFound(item_id.to_string()))
    }

    pub fn filter_by_category(&self, category: &str) -> Vec<&ProductItem> {
        self.items
            .iter()
            .filter(|i| i.category.eq_ignore_ascii_case(category))
            .collect()
    }
}
