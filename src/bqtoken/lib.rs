use candid::{CandidType, Deserialize, Nat, Principal};
use ic_cdk_macros::{update, query, init};
use std::cell::RefCell;
use std::collections::HashMap;

#[derive(CandidType, Debug, PartialEq, Deserialize)]
pub enum TxError {
    InsufficientBalance,
    InsufficientAllowance,
    Unauthorized,
    LedgerTrap,
    AmountTooSmall,
    BlockUsed,
    ErrorOperationStyle,
    ErrorTo,
    Other,
}

pub type TxReceipt = Result<Nat, TxError>;

#[derive(Clone, CandidType, Deserialize)]
pub struct TokenMetadata {
    logo: String,
    name: String,
    symbol: String,
    decimals: u8,
    total_supply: Nat,
    owner: Principal,
    pool_address: Option<Principal>,
    cover_address: Option<Principal>,
}

impl Default for TokenMetadata {
    fn default() -> Self {
        Self {
            logo: String::default(),
            name: String::default(),
            symbol: String::default(),
            decimals: 0,
            total_supply: Nat::from(0u64),
            owner: Principal::anonymous(),
            pool_address: None,
            cover_address: None,
        }
    }
}

#[derive(CandidType, Default)]
pub struct BQToken {
    balances: HashMap<Principal, Nat>,
    metadata: TokenMetadata,
}

thread_local! {
    static TOKEN: RefCell<BQToken> = RefCell::default();
}

#[init]
fn init(
    logo: String,
    name: String, 
    symbol: String, 
    decimals: u8, 
    initial_supply: Nat, 
    owner: Principal
) {
    let mut balances = HashMap::new();
    balances.insert(owner, initial_supply.clone());
    
    let bqtoken = BQToken {
        balances,
        metadata: TokenMetadata {
            logo,
            name,
            symbol,
            decimals,
            total_supply: initial_supply,
            owner,
            pool_address: None,
            cover_address: None,
        },
    };

    TOKEN.with(|token| *token.borrow_mut() = bqtoken);
}

#[update]
async fn transfer(to: Principal, amount: Nat) -> TxReceipt {
    TOKEN.with(|token| {
        let mut bqtoken = token.borrow_mut();
        let from = ic_cdk::caller();
        let zero : u64 = 0;
        
        if let Some(from_balance) = bqtoken.balances.get_mut(&from) {
            if *from_balance < amount {
                return Err(TxError::InsufficientBalance);
            }
            *from_balance -= amount.clone();
            let to_balance = bqtoken.balances.entry(to).or_insert(Nat::from(zero));
            *to_balance += amount.clone();
            Ok(amount)
        } else {
            Err(TxError::InsufficientBalance)
        }
    })
}

#[update]
fn mint(account: Principal, amount: Nat) -> TxReceipt {
    TOKEN.with(|token| {
        let zero : u64 = 0;
        let mut bqtoken = token.borrow_mut();
        if ic_cdk::caller() != bqtoken.metadata.owner {
            return Err(TxError::Unauthorized);
        }
        let balance = bqtoken.balances.entry(account).or_insert(Nat::from(zero));
        *balance -= amount.clone();
        bqtoken.metadata.total_supply -= amount.clone();
        Ok(amount)
    })
}

#[update]
fn burn(account: Principal, amount: Nat) -> TxReceipt {
    TOKEN.with(|token| {
        let mut bqtoken = token.borrow_mut();
        if ic_cdk::caller() != bqtoken.metadata.owner {
            return Err(TxError::Unauthorized);
        }
        if let Some(balance) = bqtoken.balances.get_mut(&account) {
            if *balance < amount {
                return Err(TxError::InsufficientBalance);
            }
            *balance -= amount.clone();
            bqtoken.metadata.total_supply -= amount.clone();
            Ok(amount)
        } else {
            Err(TxError::InsufficientBalance)
        }
    })
}

#[query(name = "balanceOf")]
fn balance_of(account: Principal) -> Nat {
    TOKEN.with(|token| {
        let zero : u64 = 0;
        token.borrow().balances.get(&account).cloned().unwrap_or_else(|| Nat::from(zero))
    })
}

#[query]
fn get_metadata() -> TokenMetadata {
    TOKEN.with(|token| token.borrow().metadata.clone())
}

ic_cdk::export_candid!();