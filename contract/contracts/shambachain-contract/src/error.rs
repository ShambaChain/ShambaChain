use soroban_sdk::contracterror;

#[contracterror]
#[derive(Debug, Clone, PartialEq, Eq, Copy)]
pub enum ContractError {
    // Auth 
    Unauthorized = 1,
    AlreadyInitialized = 2,

    // Farmer
    FarmerNotFound = 3,
    FarmerAlreadyExists = 4,

    // Entry
    InvalidEntryType = 5,
    InvalidAmount = 6,

    // Loan
    LoanNotFound = 7,
    ActiveLoanExists = 8,
    ScoreTooLow = 9, 
    AmountExceedsLimit = 10,
    LoanNotPending = 11,
    LoanNotDisbursed = 12,
    NotLoanLender = 13,
    OverRepayment = 14,

    // General 
    NotInitialized = 15,
}
