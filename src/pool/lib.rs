use candid::{CandidType, Deserialize, Nat, Principal};
use ic_cdk_macros::*;
use std::cell::RefCell;
use std::collections::HashMap;
use ic_cdk::api::call::call;

mod types;
use types::{Cover, Proposal, ProposalStatus, RiskType, Pool, PoolInfo, Status, Deposit};

const ZER0 :u64 = 0;

thread_local! {
    static STATE: RefCell<State> = RefCell::default();
}

#[derive(CandidType, Deserialize, Default)]
struct State {
    pools: HashMap<Nat, Pool>,
    pool_count: Nat,
    owner: Option<Principal>,
    bq_btc_address: Option<Principal>,
    cover_address: Option<Principal>,
    gov_address: Option<Principal>,
    participants: Vec<Principal>,
    participation: HashMap<Principal, Nat>,
    pool_covers: HashMap<Nat, Vec<Cover>>
}

#[init]
fn init(owner: Principal, bq_btc: Principal) {
    STATE.with(|state| {
        let mut state = state.borrow_mut();
        state.owner = Some(owner);
        state.bq_btc_address = Some(bq_btc);
    });
}

#[update(name = "createPool")]
async fn create_pool(risk_type: RiskType, pool_name: String, apy: Nat, min_period: Nat) -> Result<(), String> {
    let caller = ic_cdk::caller();
    STATE.with(|state| {
        let mut state = state.borrow_mut();
        if state.owner != Some(caller) {
            return Err("Only owner can create pools".to_string());
        }
        let pool_count = state.pool_count.clone() + Nat::from(1u64);
        let new_pool = Pool {
            pool_name,
            risk_type,
            apy,
            min_period,
            tvl: Nat::from(ZER0),
            tcp: Nat::from(ZER0),
            is_active: true,
            percentage_split_balance: Nat::from(100u64),
            deposits: HashMap::new(),
        };
        state.pools.insert(pool_count.clone(), new_pool);
        state.pool_count = pool_count;
        Ok(())
    })
}

#[update(name = "updatePool")]
async fn update_pool(pool_id: Nat, apy: Nat, min_period: Nat) -> Result<(), String> {
    let caller = ic_cdk::caller();
    STATE.with(|state| {
        let mut state = state.borrow_mut();
        if state.owner != Some(caller) {
            return Err("Only owner can create pools".to_string());
        }

        let pool = state.pools.get_mut(&pool_id).ok_or("Pool not found".to_string())?;
        pool.apy = apy;
        pool.min_period = min_period;
        Ok(())
    })
}

#[update(name = "reducePercentageSplit")]
async fn reduce_percentage_split(pool_id: Nat, percentage_split: Nat) -> Result<(), String> {
    let caller = ic_cdk::caller();
    STATE.with(|state| {
        let mut state = state.borrow_mut();
        if state.cover_address != Some(caller) {
            return Err("Only cover contract can create pools".to_string());
        }

        let pool = state.pools.get_mut(&pool_id).ok_or("Pool not found".to_string())?;
        pool.percentage_split_balance -= percentage_split;
        Ok(())
    })
}

#[update(name = "increasePercentageSplit")]
async fn increase_percentage_split(pool_id: Nat, percentage_split: Nat) -> Result<(), String> {
    let caller = ic_cdk::caller();
    STATE.with(|state| {
        let mut state = state.borrow_mut();
        if state.cover_address != Some(caller) {
            return Err("Only cover contract can create pools".to_string());
        }

        let pool = state.pools.get_mut(&pool_id).ok_or("Pool not found".to_string())?;
        pool.percentage_split_balance += percentage_split;
        Ok(())
    })
}

#[update(name = "deactivatePool")]
async fn deactivate_pool(pool_id: Nat) -> Result<(), String> {
    let caller = ic_cdk::caller();
    STATE.with(|state| {
        let mut state = state.borrow_mut();
        if state.owner != Some(caller) {
            return Err("Only owner can create pools".to_string());
        }

        let pool = state.pools.get_mut(&pool_id).ok_or("Pool not found".to_string())?;
        pool.is_active = false;
        Ok(())
    })
}

#[query(name = "getPool")]
fn get_pool(pool_id: Nat) -> Result<Pool, String> {
    STATE.with(|state| {
        let state = state.borrow();
        state.pools.get(&pool_id).cloned().ok_or("Pool not found".to_string())
    })
}

#[query(name = "getAllPools")]
fn get_all_pools() -> Vec<(Nat, Pool)> {
    STATE.with(|state| {
        let state = state.borrow();
        state.pools.iter().map(|(k, v)| (k.clone(), v.clone())).collect()
    })
}

#[update(name = "updatePoolCovers")]
async fn update_pool_covers(pool_id: Nat, new_cover: Cover) -> Result<(), String> {
    let caller = ic_cdk::caller();
    STATE.with(|state| {
        let mut state = state.borrow_mut();
        if state.cover_address != Some(caller) {
            return Err("Only the cover contract can update covers".to_string());
        }

        let pool_covers = state.pool_covers.get_mut(&pool_id).ok_or("Pool not found".to_string())?;
        let mut found = false;

        for cover in pool_covers.iter_mut() {
            if cover.id == new_cover.id {
                *cover = new_cover;
                found = true;
                break;
            }
        }

        if !found {
            return Err("Cover not found in pool".to_string());
        }

        Ok(())
    })
}

#[update(name = "addPoolCover")]
async fn add_pool_covers(pool_id: Nat, new_cover: Cover) -> Result<(), String> {
    let caller = ic_cdk::caller();
    STATE.with(|state| {
        let mut state = state.borrow_mut();
        if state.cover_address != Some(caller) {
            return Err("Only the cover contract can add covers".to_string());
        }

        let pool_covers = state.pool_covers.get_mut(&pool_id).ok_or("Pool not found".to_string())?;
        
        pool_covers.push(new_cover);
        
        Ok(())
    })
}

#[query(name = "getPoolCovers")]
async fn get_pool_covers(pool_id: Nat) -> Result<Vec<Cover>, String> {
    STATE.with(|state| {
        let state = state.borrow();

        let pool_covers = state.pool_covers.get(&pool_id).ok_or("Pool not found".to_string())?;
        
        Ok(pool_covers.clone())
    })
}

#[query(name = "getPoolsByAddress")]
fn get_pools_by_address(user_address: Principal) -> Vec<PoolInfo> {
    STATE.with(|state| {
        let state = state.borrow();
        state.pools.iter()
            .filter_map(|(pool_id, pool)| {
                pool.deposits.get(&user_address).map(|deposit| {
                    PoolInfo {
                        pool_name: pool.pool_name.clone(),
                        pool_id: pool_id.clone(),
                        daily_payout: deposit.daily_payout.clone(),
                        deposit_amount: deposit.amount.clone(),
                        apy: pool.apy.clone(),
                        min_period: pool.min_period.clone(),
                        tvl: pool.tvl.clone(),
                        tcp: pool.tcp.clone(),
                        is_active: pool.is_active,
                        accrued_payout: deposit.accrued_payout.clone(),
                    }
                })
            })
            .collect()
    })
}

#[update(name = "withdraw")]
async fn withdraw(pool_id: Nat, amount: Nat) -> Result<(), String> {
    let caller = ic_cdk::caller();

    let (amount_to_mint, bq_btc_address) = STATE.with(|state| {
        let mut state = state.borrow_mut();
        let bq_btc_address = state.bq_btc_address.ok_or("bqBTC canister address not set".to_string())?;
        let pool = state.pools.get_mut(&pool_id).ok_or("Pool not found")?;

        if !pool.is_active {
            return Err("Pool is inactive".to_string());
        }

        if let Some(caller_deposit) = pool.deposits.get_mut(&caller) {
            let current_time = Nat::from(ic_cdk::api::time() / 1_000_000_000);
            if current_time < caller_deposit.expiry_date {
                return Err("Cant withdraw before the end of a deposit period".to_string());
            }
            if caller_deposit.status == Status::Withdrawn {
                return Err("Caller has already withdrawn".to_string());
            } 
            if caller_deposit.amount == Nat::from(0u64) {
                return Err("Caller deposit is 0".to_string());
            }
            if amount > caller_deposit.amount {
                return Err("Amount is more than caller deposit".to_string());
            }

            caller_deposit.amount -= amount.clone();
            pool.tvl -= amount.clone();
            Ok((amount.clone(), bq_btc_address))
        } else {
            Err("No deposit found for caller".to_string())
        }
    })?;

    let mint_result: Result<(), _> = call(bq_btc_address, "mint", (caller, amount_to_mint)).await;

    mint_result.map_err(|err| format!("Error minting BQ BTC: {:?}", err))
}

#[update(name = "deposit")]
async fn deposit(pool_id: Nat, amount: Nat) -> Result<(), String> {
    let caller = ic_cdk::caller();

    let (bq_btc_address, daily_payout, min_period) = STATE.with(|state| {
        let state = state.borrow();
        let pool = state.pools.get(&pool_id).ok_or("Pool not found")?;
        
        if !pool.is_active {
            return Err("Pool is inactive".to_string());
        }

        let bq_btc_address = state.bq_btc_address.ok_or("bqBTC canister address not set".to_string())?;
        let daily_payout = (amount.clone() * pool.apy.clone()) / Nat::from(100u64) / Nat::from(365u64);
        let min_period = pool.min_period.clone();

        Ok((bq_btc_address, daily_payout, min_period))
    })?;

    let burn_result: Result<(), _> = call(bq_btc_address, "burn", (caller, amount.clone())).await;

    match burn_result {
        Ok(_) => {
            STATE.with(|state| {
                let mut state = state.borrow_mut();
                let pool = state.pools.get_mut(&pool_id).expect("Pool should be found");
                pool.tvl += amount.clone();

                let deposit = Deposit {
                    lp: caller,
                    amount: amount.clone(),
                    pool_id: pool_id.clone(),
                    daily_payout,
                    status: Status::Active,
                    days_left: min_period.clone(),
                    start_date: Nat::from(ic_cdk::api::time() / 1_000_000_000),
                    expiry_date: Nat::from(ic_cdk::api::time() / 1_000_000_000) + min_period * Nat::from(86400u64),
                    accrued_payout: Nat::from(0u64),
                };

                pool.deposits.insert(caller, deposit);

                if !state.participants.contains(&caller) {
                    state.participants.push(caller);
                }
                *state.participation.entry(caller).or_insert(Nat::from(0u64)) += Nat::from(1u64);
            });

            Ok(())
        },
        Err(err) => Err(format!("Error burning BQ BTC: {:?}", err)),
    }
}

#[update(name="claimProposalFunds")] 
pub async fn claim_proposal_funds(proposal_id: Nat) -> Result<(), String> { 
    let caller = ic_cdk::caller(); 
    let (gov_canister, bqbtc_canister, cover_canister) = STATE.with(|state| { 
        let state = state.borrow(); 
        let gov_canister = state.gov_address.ok_or("Governance canister address not set")?; 
        let bqbtc_canister = state.bq_btc_address.ok_or("bqBTC canister address not set")?; 
        let cover_canister = state.cover_address.ok_or("Cover canister address not set")?; 
        Ok::<(Principal, Principal, Principal), String>((gov_canister, bqbtc_canister, cover_canister)) 
    })?; 
 
    let proposal_details: Result<(Proposal,), _> = call(gov_canister, "getProposalDetails", (caller, proposal_id.clone())).await; 
    let (proposal, ) = proposal_details.map_err(|_| "Failed to get proposal details")?; 
    if proposal.status != ProposalStatus::Approved || proposal.executed { 
        return Err("Proposal not approved".to_string()); 
    } 
    if proposal.proposal_param.user != caller { 
        return Err("Not a valid proposal".to_string()); 
    } 
 
    let _ = STATE.with(|state | { 
        let mut state = state.borrow_mut(); 
        let pool = state.pools.get_mut(&proposal.proposal_param.pool_id).ok_or("Pool should be found")?; 
 
        if !pool.is_active { 
            return Err("Pool is not active".to_string()); 
        } 
        if pool.tvl < proposal.proposal_param.claim_amount { 
            return Err("Not enough funds in the pool".to_string()); 
        } 
         
        pool.tcp += proposal.proposal_param.claim_amount.clone(); 
        pool.tvl -= proposal.proposal_param.claim_amount.clone(); 
 
        Ok::<(), String>(()) 
    }); 
 
    let pool_covers = get_pool_covers(proposal.proposal_param.pool_id).await.map_err(|_| "Error getting pool covers")?; 
    for cover in pool_covers.iter() {
        let update_result: Result<((),), _> = call(
            cover_canister, 
            "updateMaxAmount", 
            (caller, cover.id.clone())
        ).await;
        update_result.map_err(|_| format!("Failed to update cover id {}", cover.id))?;
    } 
 
    let update_status: Result<((),), _> = call(
        gov_canister, 
        "updateProposalStatusToClaimed", 
        (caller, proposal_id)
    ).await;
    update_status.map_err(|_| "Failed to update proposal status")?;

    let mint_result: Result<((),), _> = call(
        bqbtc_canister, 
        "mint", 
        (caller, proposal.proposal_param.claim_amount)
    ).await;
    mint_result.map_err(|_| "Error minting BQ BTC")?;
 
    Ok(()) 
}

#[query(name = "getUserDeposit")]
pub async fn get_user_deposit(pool_id: Nat, user: Principal) -> Result<Deposit, String> {
    STATE.with(| state | {
        let state = state.borrow();
        let pool = state.pools.get(&pool_id).ok_or("Pool should be found")?;
        
        let user_deposit = pool.deposits.get(&user).ok_or("User deposit not found")?;

        Ok(user_deposit.clone())
    })
}

#[query(name = "getPoolTVL")]
pub async fn get_pool_tvl(pool_id: Nat) -> Result<Nat, String> {
    STATE.with(| state | {
        let state = state.borrow();
        let pool = state.pools.get(&pool_id).ok_or("Pool should be found")?;
        
        let tvl = pool.tvl.clone();

        Ok(tvl)
    })
}

#[query(name = "poolActive")]
pub async fn pool_active(pool_id: Nat) -> Result<bool, String> {
    STATE.with(| state | {
        let state = state.borrow();
        let pool = state.pools.get(&pool_id).ok_or("Pool should be found")?;
        
        let active = pool.is_active;

        Ok(active)
    })
}

#[query(name = "getAllParticipants")]
pub async fn get_all_participants() -> Result<Vec<Principal>, String> {
    STATE.with(| state | {
        let state = state.borrow();
        let participants = state.participants.clone();

        Ok(participants)
    })
}

#[query(name = "getUserParticipation")]
pub async fn get_user_participation(user: Principal) -> Result<Nat, String> {
    STATE.with(| state | {
        let state = state.borrow();
        let participation = state.participation.get(&user).ok_or("error getting user participation")?;

        Ok(participation.clone())
    })
}

#[query(name = "getOwner")]
fn get_owner() -> Option<Principal> {
    STATE.with(|state| state.borrow().owner)
}

#[update(name = "setOwner")]
fn set_owner(new_owner: Principal) -> Result<(), String> {
    let caller = ic_cdk::caller();
    STATE.with(|state| {
        let mut state = state.borrow_mut();
        if state.owner != Some(caller) {
            return Err("Only the current owner can set a new owner".to_string());
        }
        state.owner = Some(new_owner);
        Ok(())
    })
}

ic_cdk::export_candid!();

// candid::export_service!();
// #[query(name = "__get_candid_interface_tmp_hack")]
// fn export_candid() -> String {
//     __export_service()
// }
