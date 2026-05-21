//! Inventory system: items, equip/unequip, pickup.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ItemDef {
    pub id: String,
    pub name: String,
    pub item_type: ItemType,
    pub rarity: Rarity,
    pub stackable: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ItemType {
    Weapon,
    Armor,
    Consumable,
    Material,
    Key,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Rarity {
    Common,
    Uncommon,
    Rare,
    Epic,
    Legendary,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InventorySlot {
    pub item: ItemDef,
    pub quantity: u32,
    pub equipped: bool,
}

/// Inventory component for hecs entity.
#[derive(Debug, Clone, Default)]
pub struct Inventory {
    pub slots: Vec<InventorySlot>,
    pub max_slots: usize,
}

impl Inventory {
    pub fn new(max_slots: usize) -> Self {
        Self {
            slots: Vec::new(),
            max_slots,
        }
    }

    /// Add item. Returns true if added, false if full.
    pub fn add_item(&mut self, item: ItemDef, quantity: u32) -> bool {
        // Stack if same item and stackable
        if item.stackable {
            for slot in &mut self.slots {
                if slot.item.id == item.id {
                    slot.quantity += quantity;
                    return true;
                }
            }
        }
        if self.slots.len() >= self.max_slots {
            return false;
        }
        self.slots.push(InventorySlot {
            item,
            quantity,
            equipped: false,
        });
        true
    }

    /// Remove item by id. Returns removed quantity.
    pub fn remove_item(&mut self, item_id: &str, quantity: u32) -> u32 {
        let mut removed = 0;
        self.slots.retain_mut(|slot| {
            if slot.item.id == item_id && removed < quantity {
                let take = slot.quantity.min(quantity - removed);
                slot.quantity -= take;
                removed += take;
                slot.quantity > 0
            } else {
                true
            }
        });
        removed
    }

    /// Equip item by id.
    pub fn equip(&mut self, item_id: &str) -> bool {
        for slot in &mut self.slots {
            if slot.item.id == item_id {
                slot.equipped = true;
                return true;
            }
        }
        false
    }

    /// Count total items.
    pub fn total_items(&self) -> u32 {
        self.slots.iter().map(|s| s.quantity).sum()
    }
}

/// Predefined items for demo.
pub fn demo_items() -> Vec<ItemDef> {
    vec![
        ItemDef {
            id: "gem-blue".into(),
            name: "Blue Gem".into(),
            item_type: ItemType::Material,
            rarity: Rarity::Rare,
            stackable: true,
        },
        ItemDef {
            id: "sword-iron".into(),
            name: "Iron Sword".into(),
            item_type: ItemType::Weapon,
            rarity: Rarity::Common,
            stackable: false,
        },
        ItemDef {
            id: "potion-hp".into(),
            name: "Health Potion".into(),
            item_type: ItemType::Consumable,
            rarity: Rarity::Common,
            stackable: true,
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inventory_add_remove() {
        let mut inv = Inventory::new(10);
        let item = demo_items()[0].clone();
        assert!(inv.add_item(item.clone(), 5));
        assert_eq!(inv.total_items(), 5);

        // Stack
        assert!(inv.add_item(item, 3));
        assert_eq!(inv.total_items(), 8);

        // Remove
        assert_eq!(inv.remove_item("gem-blue", 3), 3);
        assert_eq!(inv.total_items(), 5);
    }
}
