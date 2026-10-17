use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
pub enum InteractiveType {
    Button,
    List,
    Product,
    ProductList,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct ReplyButton {
    pub id: String,
    pub title: String,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct ListRow {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct ListSection {
    pub title: String,
    pub rows: Vec<ListRow>,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct InteractiveHeader {
    pub header_type: String, // "text" or "image_cid"
    pub content: String,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub enum InteractiveAction {
    Buttons {
        buttons: Vec<ReplyButton>,
    },
    List {
        button_text: String,
        sections: Vec<ListSection>,
    },
    Product {
        catalog_id: String,
        product_retailer_id: String,
    },
    ProductList {
        catalog_id: String,
        sections: Vec<ListSection>,
    },
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct InteractiveMessage {
    pub interactive_type: InteractiveType,
    pub header: Option<InteractiveHeader>,
    pub body_text: String,
    pub footer_text: Option<String>,
    pub action: InteractiveAction,
}

impl InteractiveMessage {
    /// Creates a quick-reply button message (up to 3 buttons)
    pub fn new_button_message(
        body: &str,
        buttons: Vec<(&str, &str)>,
        header: Option<&str>,
        footer: Option<&str>,
    ) -> Self {
        let reply_buttons = buttons
            .into_iter()
            .take(3)
            .map(|(id, title)| ReplyButton {
                id: id.to_string(),
                title: title.to_string(),
            })
            .collect();

        Self {
            interactive_type: InteractiveType::Button,
            header: header.map(|h| InteractiveHeader {
                header_type: "text".to_string(),
                content: h.to_string(),
            }),
            body_text: body.to_string(),
            footer_text: footer.map(str::to_string),
            action: InteractiveAction::Buttons {
                buttons: reply_buttons,
            },
        }
    }

    /// Creates a structured list picker message
    pub fn new_list_message(
        body: &str,
        button_text: &str,
        sections: Vec<ListSection>,
        header: Option<&str>,
        footer: Option<&str>,
    ) -> Self {
        Self {
            interactive_type: InteractiveType::List,
            header: header.map(|h| InteractiveHeader {
                header_type: "text".to_string(),
                content: h.to_string(),
            }),
            body_text: body.to_string(),
            footer_text: footer.map(str::to_string),
            action: InteractiveAction::List {
                button_text: button_text.to_string(),
                sections,
            },
        }
    }

    /// Creates a single product card message
    pub fn new_product_message(
        body: &str,
        catalog_id: &str,
        product_retailer_id: &str,
        image_cid: Option<&str>,
        footer: Option<&str>,
    ) -> Self {
        Self {
            interactive_type: InteractiveType::Product,
            header: image_cid.map(|cid| InteractiveHeader {
                header_type: "image_cid".to_string(),
                content: cid.to_string(),
            }),
            body_text: body.to_string(),
            footer_text: footer.map(str::to_string),
            action: InteractiveAction::Product {
                catalog_id: catalog_id.to_string(),
                product_retailer_id: product_retailer_id.to_string(),
            },
        }
    }
}
