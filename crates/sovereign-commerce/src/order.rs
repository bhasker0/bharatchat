use serde::{Deserialize, Serialize};
use sovereign_upi::UpiPaymentOrder;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum OrderError {
    #[error("Order has no items")]
    EmptyCart,

    #[error("Order cannot transition from {0:?} to {1:?}")]
    InvalidStatusTransition(OrderStatus, OrderStatus),

    #[error("UPI payment failed or mismatch: {0}")]
    UpiError(String),
}

pub type Result<T> = std::result::Result<T, OrderError>;

#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Debug)]
pub enum OrderStatus {
    Draft,
    AwaitingPayment,
    PaidConfirmed,
    Shipped,
    Delivered,
    Cancelled,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct CartItem {
    pub product_id: String,
    pub name: String,
    pub unit_price_inr: f64,
    pub quantity: u32,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct TaxBreakdown {
    pub gst_rate_percent: f64,
    pub cgst_inr: f64,
    pub sgst_inr: f64,
    pub total_tax_inr: f64,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct OrderDraft {
    pub order_id: String,
    pub customer_did: String,
    pub merchant_did: String,
    pub items: Vec<CartItem>,
    pub subtotal_inr: f64,
    pub tax: TaxBreakdown,
    pub total_amount_inr: f64,
    pub status: OrderStatus,
    pub created_at: u64,
    pub upi_payment_order: Option<UpiPaymentOrder>,
}

impl OrderDraft {
    pub fn new(order_id: &str, customer_did: &str, merchant_did: &str, timestamp: u64) -> Self {
        Self {
            order_id: order_id.to_string(),
            customer_did: customer_did.to_string(),
            merchant_did: merchant_did.to_string(),
            items: Vec::new(),
            subtotal_inr: 0.0,
            tax: TaxBreakdown {
                gst_rate_percent: 0.0,
                cgst_inr: 0.0,
                sgst_inr: 0.0,
                total_tax_inr: 0.0,
            },
            total_amount_inr: 0.0,
            status: OrderStatus::Draft,
            created_at: timestamp,
            upi_payment_order: None,
        }
    }

    pub fn add_item(&mut self, product_id: &str, name: &str, unit_price_inr: f64, quantity: u32) {
        if let Some(existing) = self.items.iter_mut().find(|i| i.product_id == product_id) {
            existing.quantity += quantity;
        } else {
            self.items.push(CartItem {
                product_id: product_id.to_string(),
                name: name.to_string(),
                unit_price_inr,
                quantity,
            });
        }
    }

    /// Recalculates subtotal, GST (CGST + SGST in India), and grand total
    pub fn calculate_totals(&mut self, gst_rate_percent: f64) {
        let subtotal: f64 = self
            .items
            .iter()
            .map(|i| i.unit_price_inr * (i.quantity as f64))
            .sum();

        let total_tax = subtotal * (gst_rate_percent / 100.0);
        let half_tax = total_tax / 2.0;

        self.subtotal_inr = (subtotal * 100.0).round() / 100.0;
        self.tax = TaxBreakdown {
            gst_rate_percent,
            cgst_inr: (half_tax * 100.0).round() / 100.0,
            sgst_inr: (half_tax * 100.0).round() / 100.0,
            total_tax_inr: (total_tax * 100.0).round() / 100.0,
        };
        self.total_amount_inr = ((subtotal + total_tax) * 100.0).round() / 100.0;
    }

    /// Links this order to an official NPCI UPI 2.0 Payment Intent
    pub fn generate_upi_intent(
        &mut self,
        payee_vpa: &str,
        payee_name: &str,
        timestamp: u64,
    ) -> Result<String> {
        if self.items.is_empty() || self.total_amount_inr <= 0.0 {
            return Err(OrderError::EmptyCart);
        }

        let note = format!("Order #{}", self.order_id);
        let upi_order = UpiPaymentOrder::new(
            payee_vpa,
            payee_name,
            self.total_amount_inr,
            &note,
            timestamp,
        )
        .map_err(|e| OrderError::UpiError(e.to_string()))?;

        let uri = upi_order.generate_upi_uri();
        self.upi_payment_order = Some(upi_order);
        self.status = OrderStatus::AwaitingPayment;
        Ok(uri)
    }

    /// Marks the order as paid upon UPI transaction confirmation
    pub fn confirm_payment(&mut self, timestamp: u64) -> Result<()> {
        if self.status != OrderStatus::AwaitingPayment {
            return Err(OrderError::InvalidStatusTransition(
                self.status,
                OrderStatus::PaidConfirmed,
            ));
        }

        if let Some(ref mut upi) = self.upi_payment_order {
            upi.finalize(true, timestamp)
                .map_err(|e| OrderError::UpiError(e.to_string()))?;
        }

        self.status = OrderStatus::PaidConfirmed;
        Ok(())
    }
}
