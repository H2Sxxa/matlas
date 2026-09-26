use serde::{Deserialize, Serialize};

// Lifetime totals that goals are measured against.
//
// The atlas records what exists right now; stats record what the factory has done.
// A sold item leaves the atlas unchanged, so revenue cannot be reconstructed from it.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Stats {
    crafted: usize,
    revenue: f64,
}

impl Stats {
    pub fn new() -> Self {
        Self {
            crafted: 0,
            revenue: 0.0,
        }
    }

    pub fn crafted(&self) -> usize {
        self.crafted
    }

    // Revenue is kept fractional so a price multiplier can pay out on cheap items
    // instead of being rounded away. Readers see the rounded total.
    pub fn revenue(&self) -> usize {
        self.revenue.round() as usize
    }

    pub fn record_craft(&mut self) {
        self.crafted += 1;
    }

    pub fn record_revenue(&mut self, amount: f64) {
        self.revenue += amount;
    }
}

impl Default for Stats {
    fn default() -> Self {
        Self::new()
    }
}
