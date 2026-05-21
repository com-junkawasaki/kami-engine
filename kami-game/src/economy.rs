//! Economy system: Gems wallet + transactions.

use serde::{Deserialize, Serialize};

/// Wallet component for hecs entity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Wallet {
    pub gems: i64,
    pub transactions: Vec<Transaction>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub amount: i64,
    pub reason: String,
    pub tick: u32,
}

impl Wallet {
    pub fn new(initial_gems: i64) -> Self {
        Self {
            gems: initial_gems,
            transactions: Vec::new(),
        }
    }

    /// Credit gems. Returns new balance.
    pub fn credit(&mut self, amount: i64, reason: &str, tick: u32) -> i64 {
        self.gems += amount;
        self.transactions.push(Transaction {
            amount,
            reason: reason.to_string(),
            tick,
        });
        self.gems
    }

    /// Debit gems. Returns Ok(new_balance) or Err if insufficient.
    pub fn debit(&mut self, amount: i64, reason: &str, tick: u32) -> Result<i64, &'static str> {
        if self.gems < amount {
            return Err("insufficient gems");
        }
        self.gems -= amount;
        self.transactions.push(Transaction {
            amount: -amount,
            reason: reason.to_string(),
            tick,
        });
        Ok(self.gems)
    }
}

impl Default for Wallet {
    fn default() -> Self {
        Self::new(100) // starting gems
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wallet_credit_debit() {
        let mut w = Wallet::new(100);
        assert_eq!(w.credit(50, "quest reward", 1), 150);
        assert_eq!(w.debit(30, "shop purchase", 2).unwrap(), 120);
        assert!(w.debit(200, "too expensive", 3).is_err());
        assert_eq!(w.gems, 120);
    }
}
