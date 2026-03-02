use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct InventoryDoc {
    #[serde(default = "default_version")]
    pub version: u32,
    #[serde(default)]
    pub items: Vec<Item>,
}

impl Default for InventoryDoc {
    fn default() -> Self {
        Self {
            version: default_version(),
            items: Vec::new(),
        }
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Item {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub quantity: u64,
    pub unit: String,
    pub location: Option<String>,
    pub bin_size: Option<String>,
    pub supplier: Option<String>,
    pub source_url: Option<String>,
    pub manufacturer: Option<String>,
    pub mpn: Option<String>,
    pub tags: Vec<String>,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Deserialize)]
struct ItemWire {
    id: Uuid,
    name: String,
    #[serde(default)]
    description: Option<String>,
    #[serde(default)]
    quantity: u64,
    #[serde(default = "default_unit")]
    unit: String,
    #[serde(default)]
    location: Option<String>,
    #[serde(default)]
    bin_size: Option<String>,
    #[serde(default)]
    supplier: Option<String>,
    #[serde(default)]
    source_url: Option<String>,
    #[serde(default)]
    manufacturer: Option<String>,
    #[serde(default)]
    mpn: Option<String>,
    #[serde(default)]
    tags: Vec<String>,
    #[serde(default)]
    notes: Option<String>,
    #[serde(default)]
    created_at: Option<DateTime<Utc>>,
    #[serde(default)]
    updated_at: Option<DateTime<Utc>>,
}

impl<'de> Deserialize<'de> for Item {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let wire = ItemWire::deserialize(deserializer)?;
        let now = Utc::now();
        let created_at = wire.created_at.unwrap_or(now);
        let updated_at = wire.updated_at.unwrap_or(created_at);

        Ok(Self {
            id: wire.id,
            name: wire.name,
            description: wire.description,
            quantity: wire.quantity,
            unit: wire.unit,
            location: wire.location,
            bin_size: wire.bin_size,
            supplier: wire.supplier,
            source_url: wire.source_url,
            manufacturer: wire.manufacturer,
            mpn: wire.mpn,
            tags: wire.tags,
            notes: wire.notes,
            created_at,
            updated_at,
        })
    }
}

#[allow(dead_code)]
impl Item {
    pub fn with_required_fields(id: Uuid, name: impl Into<String>) -> Self {
        let now = Utc::now();

        Self {
            id,
            name: name.into(),
            description: None,
            quantity: 0,
            unit: default_unit(),
            location: None,
            bin_size: None,
            supplier: None,
            source_url: None,
            manufacturer: None,
            mpn: None,
            tags: Vec::new(),
            notes: None,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn refresh_updated_at(&mut self) {
        self.updated_at = Utc::now();
    }
}

fn default_version() -> u32 {
    1
}

fn default_unit() -> String {
    "pcs".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn serde_roundtrip_inventory_doc_with_item() {
        let item_id = Uuid::new_v4();
        let mut item = Item::with_required_fields(item_id, "ESP32 Dev Board");
        item.description = Some("Wi-Fi + BLE microcontroller".to_string());
        item.quantity = 3;
        item.location = Some("Shelf A".to_string());
        item.bin_size = Some("small".to_string());
        item.supplier = Some("LCSC".to_string());
        item.source_url = Some("https://example.com/esp32".to_string());
        item.manufacturer = Some("Espressif".to_string());
        item.mpn = Some("ESP32-WROOM-32".to_string());
        item.tags = vec!["mcu".to_string(), "wifi".to_string()];
        item.notes = Some("Great for prototyping".to_string());

        let doc = InventoryDoc {
            version: 1,
            items: vec![item],
        };

        let serialized = serde_json::to_value(&doc).expect("serialize should work");
        let deserialized: InventoryDoc =
            serde_json::from_value(serialized.clone()).expect("deserialize should work");

        assert_eq!(deserialized.version, 1);
        assert_eq!(deserialized.items.len(), 1);
        assert_eq!(deserialized.items[0].id, item_id);
        assert_eq!(serialized["items"][0]["id"], json!(item_id.to_string()));
    }

    #[test]
    fn serde_inventory_doc_deserialize_applies_defaults() {
        let value = json!({});

        let doc: InventoryDoc = serde_json::from_value(value).expect("deserialize should work");

        assert_eq!(doc.version, 1);
        assert!(doc.items.is_empty());
    }

    #[test]
    fn serde_item_deserialize_applies_defaults() {
        let value = json!({
            "id": Uuid::new_v4().to_string(),
            "name": "Resistor pack"
        });

        let item: Item = serde_json::from_value(value).expect("deserialize should work");

        assert_eq!(item.unit, "pcs");
        assert_eq!(item.quantity, 0);
        assert_eq!(item.created_at, item.updated_at);
        assert!(item.tags.is_empty());
    }

    #[test]
    fn serde_item_with_required_fields_sets_defaults() {
        let item = Item::with_required_fields(Uuid::new_v4(), "Capacitor kit");

        assert_eq!(item.unit, "pcs");
        assert_eq!(item.quantity, 0);
        assert!(item.tags.is_empty());
        assert_eq!(item.created_at, item.updated_at);
    }

    #[test]
    fn serde_item_refresh_updated_at_changes_timestamp() {
        let mut item = Item::with_required_fields(Uuid::new_v4(), "Breadboard");
        let before = item.updated_at;

        thread::sleep(Duration::from_millis(2));
        item.refresh_updated_at();

        assert!(item.updated_at > before);
    }

    #[test]
    fn serde_item_deserialize_rejects_invalid_uuid() {
        let value = json!({
            "id": "not-a-uuid",
            "name": "Invalid"
        });

        let result: serde_json::Result<Item> = serde_json::from_value(value);

        assert!(result.is_err());
    }
}
