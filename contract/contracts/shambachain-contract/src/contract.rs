use soroban_sdk::{contract, contractimpl, Address, Env, String, Vec};

use crate::{
    error::ContractError,
    events::Events,
    score,
    storage::{DataKey, EntryKey},
    trait_def::ShambaChainInterface,
    types::{
        CreditTier, EntryType, FarmerProfile, LedgerEntry, LoanApplication, LoanStatus,
    },
};

#[contract]
pub struct ShambaChain;

#[contractimpl]
impl ShambaChainInterface for ShambaChain {
    // ── Lifecycle ─────────────────────────────────────────────────────────────

    fn initialize(env: Env, admin: Address) -> Result<(), ContractError> {
        if env.storage().instance().has(&DataKey::Admin) {
            return Err(ContractError::AlreadyInitialized);
        }
        admin.require_auth();
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::LoanCounter, &0u64);
        Ok(())
    }

    // ── Farmer ────────────────────────────────────────────────────────────────

    fn register_farmer(
        env:             Env,
        farmer:          Address,
        name:            String,
        county:          String,
        primary_crop:    String,
        farm_size_acres: u32,
    ) -> Result<FarmerProfile, ContractError> {
        farmer.require_auth();

        if env.storage().persistent().has(&DataKey::FarmerProfile(farmer.clone())) {
            return Err(ContractError::FarmerAlreadyExists);
        }

        let profile = FarmerProfile {
            address:          farmer.clone(),
            name:             name.clone(),
            county:           county.clone(),
            primary_crop:     primary_crop.clone(),
            farm_size_acres,
            registered_at:    env.ledger().timestamp(),
            total_harvests:   0,
            total_sales:      0,
            total_income_kes: 0,
            credit_score:     100,
            tier:             CreditTier::Unranked,
            active_loan:      false,
            total_entries:    0,
        };

        env.storage()
            .persistent()
            .set(&DataKey::FarmerProfile(farmer.clone()), &profile);
        env.storage()
            .persistent()
            .set(&DataKey::EntryCounter(farmer.clone()), &0u64);

        // Append to global farmer list
        let mut list: Vec<Address> = env
            .storage()
            .instance()
            .get(&DataKey::FarmerList)
            .unwrap_or(Vec::new(&env));
        list.push_back(farmer.clone());
        env.storage().instance().set(&DataKey::FarmerList, &list);

        Events::farmer_registered(&env, farmer, name, county);
        Ok(profile)
    }

    fn get_farmer(env: Env, farmer: Address) -> Result<FarmerProfile, ContractError> {
        env.storage()
            .persistent()
            .get(&DataKey::FarmerProfile(farmer))
            .ok_or(ContractError::FarmerNotFound)
    }

    fn get_all_farmers(env: Env) -> Vec<Address> {
        env.storage()
            .instance()
            .get(&DataKey::FarmerList)
            .unwrap_or(Vec::new(&env))
    }

    // ── Ledger ────────────────────────────────────────────────────────────────

    fn log_entry(
        env:        Env,
        farmer:     Address,
        entry_type: EntryType,
        crop:       String,
        qty_kg:     u32,
        value_kes:  u64,
        notes:      String,
    ) -> Result<LedgerEntry, ContractError> {
        farmer.require_auth();

        let mut profile: FarmerProfile = env
            .storage()
            .persistent()
            .get(&DataKey::FarmerProfile(farmer.clone()))
            .ok_or(ContractError::FarmerNotFound)?;

        // Increment entry counter
        let mut counter: u64 = env
            .storage()
            .persistent()
            .get(&DataKey::EntryCounter(farmer.clone()))
            .unwrap_or(0);
        counter = counter.saturating_add(1);

        let entry = LedgerEntry {
            id:          counter,
            entry_type:  entry_type.clone(),
            crop:        crop.clone(),
            quantity_kg: qty_kg,
            value_kes,
            timestamp:   env.ledger().timestamp(),
            notes,
        };

        // Update profile statistics
        let type_code: u32 = match &entry_type {
            EntryType::Harvest => {
                profile.total_harvests = profile.total_harvests.saturating_add(1);
                0
            }
            EntryType::Sale => {
                profile.total_sales = profile.total_sales.saturating_add(1);
                profile.total_income_kes = profile.total_income_kes.saturating_add(value_kes);
                1
            }
            EntryType::Expense => 2,
        };
        profile.total_entries = profile.total_entries.saturating_add(1);

        // Recalculate credit score
        let old_score = profile.credit_score;
        let new_score = score::calculate(
            profile.total_harvests,
            profile.total_sales,
            profile.total_income_kes,
            profile.total_entries,
        );
        profile.credit_score = new_score;
        profile.tier         = score::to_tier(new_score);

        // Persist entry
        let ekey = DataKey::FarmerEntry(EntryKey {
            farmer: farmer.clone(),
            id:     counter,
        });
        env.storage().persistent().set(&ekey, &entry);
        env.storage()
            .persistent()
            .set(&DataKey::EntryCounter(farmer.clone()), &counter);
        env.storage()
            .persistent()
            .set(&DataKey::FarmerProfile(farmer.clone()), &profile);

        Events::entry_logged(&env, farmer.clone(), counter, type_code, value_kes, new_score);
        if old_score != new_score {
            Events::score_updated(&env, farmer, old_score, new_score);
        }

        Ok(entry)
    }

    fn get_ledger(env: Env, farmer: Address) -> Vec<LedgerEntry> {
        let counter: u64 = env
            .storage()
            .persistent()
            .get(&DataKey::EntryCounter(farmer.clone()))
            .unwrap_or(0);

        let mut entries = Vec::new(&env);
        for id in 1..=counter {
            let key = DataKey::FarmerEntry(EntryKey {
                farmer: farmer.clone(),
                id,
            });
            if let Some(e) = env.storage().persistent().get::<DataKey, LedgerEntry>(&key) {
                entries.push_back(e);
            }
        }
        entries
    }

    fn get_entry(env: Env, farmer: Address, entry_id: u64) -> Result<LedgerEntry, ContractError> {
        let key = DataKey::FarmerEntry(EntryKey {
            farmer,
            id: entry_id,
        });
        env.storage()
            .persistent()
            .get(&key)
            .ok_or(ContractError::FarmerNotFound)
    }

    // ── Credit Score ──────────────────────────────────────────────────────────

    fn get_credit_score(env: Env, farmer: Address) -> Result<u32, ContractError> {
        let profile: FarmerProfile = env
            .storage()
            .persistent()
            .get(&DataKey::FarmerProfile(farmer))
            .ok_or(ContractError::FarmerNotFound)?;
        Ok(profile.credit_score)
    }

    // ── Loans ─────────────────────────────────────────────────────────────────

    fn apply_for_loan(
        env:           Env,
        farmer:        Address,
        lender:        Address,
        amount_usdc:   u64,
        period_months: u32,
        purpose:       String,
    ) -> Result<u64, ContractError> {
        farmer.require_auth();

        let profile: FarmerProfile = env
            .storage()
            .persistent()
            .get(&DataKey::FarmerProfile(farmer.clone()))
            .ok_or(ContractError::FarmerNotFound)?;

        if profile.active_loan {
            return Err(ContractError::ActiveLoanExists);
        }
        if profile.credit_score < score::MIN_LOAN_SCORE {
            return Err(ContractError::ScoreTooLow);
        }

        let max = score::max_loan_usdc(&profile.tier);
        if amount_usdc > max {
            return Err(ContractError::AmountExceedsLimit);
        }

        let mut loan_ctr: u64 = env
            .storage()
            .instance()
            .get(&DataKey::LoanCounter)
            .unwrap_or(0);
        loan_ctr = loan_ctr.saturating_add(1);

        let loan = LoanApplication {
            id:             loan_ctr,
            farmer:         farmer.clone(),
            lender:         lender.clone(),
            amount_usdc,
            period_months,
            purpose,
            status:         LoanStatus::Pending,
            applied_at:     env.ledger().timestamp(),
            approved_at:    0,
            repaid_amount:  0,
            credit_score_at_application: profile.credit_score,
        };

        env.storage()
            .persistent()
            .set(&DataKey::LoanApp(loan_ctr), &loan);
        env.storage()
            .instance()
            .set(&DataKey::LoanCounter, &loan_ctr);
        env.storage()
            .persistent()
            .set(&DataKey::FarmerLoanId(farmer.clone()), &loan_ctr);

        Events::loan_applied(&env, farmer, loan_ctr, amount_usdc, period_months);
        Ok(loan_ctr)
    }

    fn approve_loan(
        env:     Env,
        lender:  Address,
        loan_id: u64,
    ) -> Result<LoanApplication, ContractError> {
        lender.require_auth();

        let mut loan: LoanApplication = env
            .storage()
            .persistent()
            .get(&DataKey::LoanApp(loan_id))
            .ok_or(ContractError::LoanNotFound)?;

        if loan.lender != lender {
            return Err(ContractError::NotLoanLender);
        }
        if loan.status != LoanStatus::Pending {
            return Err(ContractError::LoanNotPending);
        }

        loan.status      = LoanStatus::Disbursed;
        loan.approved_at = env.ledger().timestamp();

        // Mark farmer as having an active loan
        let mut profile: FarmerProfile = env
            .storage()
            .persistent()
            .get(&DataKey::FarmerProfile(loan.farmer.clone()))
            .ok_or(ContractError::FarmerNotFound)?;
        profile.active_loan = true;
        env.storage()
            .persistent()
            .set(&DataKey::FarmerProfile(loan.farmer.clone()), &profile);
        env.storage()
            .persistent()
            .set(&DataKey::LoanApp(loan_id), &loan);

        Events::loan_approved(&env, lender, loan_id, loan.farmer.clone());
        Ok(loan)
    }

    fn reject_loan(
        env:     Env,
        lender:  Address,
        loan_id: u64,
    ) -> Result<(), ContractError> {
        lender.require_auth();

        let mut loan: LoanApplication = env
            .storage()
            .persistent()
            .get(&DataKey::LoanApp(loan_id))
            .ok_or(ContractError::LoanNotFound)?;

        if loan.lender != lender {
            return Err(ContractError::NotLoanLender);
        }
        if loan.status != LoanStatus::Pending {
            return Err(ContractError::LoanNotPending);
        }

        loan.status = LoanStatus::Rejected;
        env.storage()
            .persistent()
            .set(&DataKey::LoanApp(loan_id), &loan);
        Ok(())
    }

    fn repay_loan(
        env:     Env,
        farmer:  Address,
        loan_id: u64,
        amount:  u64,
    ) -> Result<LoanApplication, ContractError> {
        farmer.require_auth();

        let mut loan: LoanApplication = env
            .storage()
            .persistent()
            .get(&DataKey::LoanApp(loan_id))
            .ok_or(ContractError::LoanNotFound)?;

        if loan.farmer != farmer {
            return Err(ContractError::Unauthorized);
        }
        if loan.status != LoanStatus::Disbursed {
            return Err(ContractError::LoanNotDisbursed);
        }

        let remaining = loan.amount_usdc.saturating_sub(loan.repaid_amount);
        if amount > remaining {
            return Err(ContractError::OverRepayment);
        }

        loan.repaid_amount = loan.repaid_amount.saturating_add(amount);
        let fully_repaid   = loan.repaid_amount >= loan.amount_usdc;

        if fully_repaid {
            loan.status = LoanStatus::Repaid;

            let mut profile: FarmerProfile = env
                .storage()
                .persistent()
                .get(&DataKey::FarmerProfile(farmer.clone()))
                .ok_or(ContractError::FarmerNotFound)?;
            profile.active_loan  = false;
            // Repayment bonus: +50 score capped at 850
            let boosted           = profile.credit_score.saturating_add(50).min(850);
            profile.credit_score  = boosted;
            profile.tier          = score::to_tier(boosted);
            env.storage()
                .persistent()
                .set(&DataKey::FarmerProfile(farmer.clone()), &profile);
        }

        env.storage()
            .persistent()
            .set(&DataKey::LoanApp(loan_id), &loan);

        Events::loan_repaid(&env, farmer, loan_id, amount, fully_repaid);
        Ok(loan)
    }

    fn get_loan(env: Env, loan_id: u64) -> Result<LoanApplication, ContractError> {
        env.storage()
            .persistent()
            .get(&DataKey::LoanApp(loan_id))
            .ok_or(ContractError::LoanNotFound)
    }

    fn get_farmer_loan_id(env: Env, farmer: Address) -> Option<u64> {
        env.storage()
            .persistent()
            .get(&DataKey::FarmerLoanId(farmer))
    }

    // ── Admin ─────────────────────────────────────────────────────────────────

    fn get_admin(env: Env) -> Result<Address, ContractError> {
        env.storage()
            .instance()
            .get(&DataKey::Admin)
            .ok_or(ContractError::NotInitialized)
    }
}
