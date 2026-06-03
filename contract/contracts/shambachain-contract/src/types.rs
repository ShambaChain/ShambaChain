use soroban_sdk::{contracttype,Address, Env, String};

// ----------- Credit Tier -----------
#[contracttype]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CreditTier {
    Unranked,
    Bronze,
    Silver,
    Gold,
    Platinum,
}

// ----------- Farmer Profile -----------
#[contracttype]
#[derive(Debug, Clone)]
pub struct FarmerProfile {
    pub address: Address,
    pub name: String,
    pub county: String,
    pub primary_crop: String,
    // Farm size stored as acres × 1000 (e.g. 2.5 acres -> 250)
    pub farm_size_acres: u32,  
    pub registered_at: u64,
    pub total_harvests: u32,
    pub total_sales: u32,
    // Cumulate income in local currency units (integer, currency-agnostic)
    pub total_income_kes: u64,
    pub credit_score : u32, // 0-850
    pub tier: CreditTier,
    pub active_loan: bool, 
    pub total_entries: u64,
}

// ----------- Ledger Entry Type ------------------------
#[contracttype]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EntryType {
    Harvest,
    Sale,
    Expense,
}

// ----------- Ledger Entry -----------
#[contracttype]
#[derive(Debug, Clone)]
pub struct LedgerEntry {
    pub id: u64,
    pub entry_type: EntryType,
    pub crop: String,
    pub quantity_kg: u32,
    pub value_kes: u64,
    pub timestamp: u64,
    pub notes: String,
}

// ------------ Loan Status ------------
#[contracttype]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LoanStatus {
    Pending,
    Approved,
    Disbursed,
    Repaid,
    Rejected,
}

// ------- Loan Application -----------
#[contracttype]
#[derive(Debug, Clone)]
pub struct LoanApplication {
    pub id: u64,
    pub farmer: Address,
    pub lender: Address,
    pub amount_usdc: u64,   // in USDC (integer units)
    pub period_months: u32, 
    pub purpose: String,
    pub status: LoanStatus,
    pub applied_at: u64,
    pub approved_at: u64,
    pub repaid_amount: u64,
    pub credit_score_at_application: u32,
}