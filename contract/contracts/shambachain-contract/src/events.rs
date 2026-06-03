use soroban_sdk::{symbol_short, Address, Env, String};

pub struct Events;

impl Events {
    pub fn farmer_registered(env: &Env, farmer: Address, name: String, county: String) {
        let topics = (symbol_short!("f_reg"), farmer);
        env.events().publish(topics, (name, county));
    }

    pub fn entry_logged(
        env: &Env,
        farmer: Address,
        entry_id: u64,
        entry_type: u32, //0=harvest 1=scale 2=expense
        value_kes: u64,
        new_score: u32,
    ){ 
        let topics = (symbol_short!("e_log"), farmer, entry_id);
        env.events().publish(topics, (entry_type, value_kes, new_score));
    }

    pub fn loan_applied(
        env: &Env,
        farmer: Address,
        loan_id: u64,
        amount: u64,
        period: u32,
     ) {
        let topics = (symbol_short!("l_apply"), farmer, loan_id);
        env.events().publish(topics, (amount, period));
     }


     pub fn loan_approved(env: &Env, lender: Address, loan_id: u64, farmer: Address) {
        let topics = (symbol_short!("l_appr"), lender, loan_id);
        env.events().publish(topics, farmer);
     }

     pub fn loan_repaid(env: &Env, farmer: Address, loan_id: u64, amount: u64, fully_repaid: bool) {
        let topics = (symbol_short!("l_repay"), farmer, loan_id);
        env.events().publish(topics, (amount, fully_repaid));      
     }

     pub fn score_updated(env: &Env, farmer: Address, old_score: u32, new_score: u32) {
        let topics = (symbol_short!("score"), farmer);
        env.events().publish(topics, (old_score, new_score));
     }
}