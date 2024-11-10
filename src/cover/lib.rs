use candid::{CandidType, Deserialize, Nat, Principal};
use ic_cdk_macros::*;
use std::cell::RefCell;
use std::collections::HashMap;
use ic_cdk::api::call::call;
mod types;
use types::{Cover, Deposit, GenericCoverInfo, Pool, RiskType, Status};

thread_local! {
    static STATE: RefCell<State> = RefCell::default();
}

#[derive(CandidType, Deserialize, Default)]
struct State {
    covers: HashMap<Nat, Cover>,
    cover_count: Nat,
    owner: Option<Principal>,
    bqbtc_address: Option<Principal>,
    lp_contract: Option<Principal>,
    gov_address: Option<Principal>,
    participants: Vec<Principal>,
    participation: HashMap<Principal, Nat>,
    user_covers: HashMap<Principal, HashMap<Nat, GenericCoverInfo>>,
    lp_claims: HashMap<Principal, HashMap<Nat, Nat>>,
    cover_ids: Vec<Nat>
}

#[init]
fn init(lp_contract: Principal, initial_owner: Principal, governance: Principal, bqbtc: Principal) {
    STATE.with(|state| {
        let mut state = state.borrow_mut();
        state.lp_contract = Some(lp_contract);
        state.owner = Some(initial_owner);
        state.gov_address = Some(governance);
        state.bqbtc_address = Some(bqbtc);
    });
}

#[update(name = "createCover")]
pub async fn create_cover(
    cover_id: Nat,
    cid: String,
    risk_type: RiskType,
    cover_name: String,
    chains: String,
    capacity: Nat,
    cost: Nat,
    pool_id: Nat,
) -> Result<(), String> {
    let caller =  ic_cdk::caller();
    let pool_contract = STATE.with(| state | {
        let state = state.borrow();
        state.lp_contract.unwrap()
    });

    let pool_covers: Result<(Vec<Cover>,), _> = call(pool_contract, "getPoolCovers", (caller, pool_id.clone())).await;
    let (covers,) = pool_covers.map_err(|_| "Failed to get pool covers")?;
    let pool_details: Result<(Pool,), _> = call(pool_contract, "getPool", (caller, pool_id.clone())).await;
    let (pool, ) = pool_details.map_err(|_| "Failed to get pool")?;

    for cover in covers.iter() {
        if cover.cover_name == cover_name || cover_id.clone() == cover.id {
            return Err("Cover Already exists!".to_string());
        }
    }

    if risk_type != pool.risk_type {
        return Err("Wrong pool, risk type must be the same!".to_string());
    }

    let precision = Nat::from(1_000_000_000_000_000_000u128);
    let maxamount: Nat = (pool.tvl * (capacity.clone() * precision.clone() / Nat::from(100u64))) / precision;


    STATE.with(|state| {
        let mut state = state.borrow_mut();
              
        let cover = Cover {
            id: cover_id.clone(),
            cover_name,
            risk_type,
            chains,
            capacity,
            cost,
            capacity_amount: maxamount.clone(),
            cover_values: Nat::from(0u64),
            max_amount: maxamount,
            pool_id,
            cid,
        };

        state.covers.insert(cover_id.clone(), cover);
        state.cover_ids.push(cover_id);
        Ok(())
    })
}

#[update(name = "updateCover")]
pub async fn update_cover(
    cover_id: Nat,
    cover_name: String,
    risk_type: RiskType,
    cid: String,
    chains: String,
    capacity: Nat,
    cost: Nat,
    pool_id: Nat
) -> Result<(), String> {
    let caller =  ic_cdk::caller();
    let pool_contract = STATE.with(| state | {
        let state = state.borrow();
        state.lp_contract.unwrap()
    });

    let pool_covers: Result<(Vec<Cover>,), _> = call(pool_contract, "getPoolCovers", (caller, pool_id.clone())).await;
    let (covers,) = pool_covers.map_err(|_| "Failed to get pool covers")?;
    let pool_details: Result<(Pool,), _> = call(pool_contract, "getPool", (caller, pool_id.clone())).await;
    let (pool, ) = pool_details.map_err(|_| "Failed to get pool")?;

    for cover in covers.iter() {
        if cover.cover_name == cover_name || cover_id.clone() != cover.id {
            return Err("Cover Already exists!".to_string());
        }
    }

    if risk_type != pool.risk_type {
        return Err("Wrong pool, risk type must be the same!".to_string());
    }

    let precision = Nat::from(1_000_000_000_000_000_000u128);
    let maxamount: Nat = (pool.tvl * (capacity.clone() * precision.clone() / Nat::from(100u64))) / precision;

    let old_capacity = STATE.with(|state| {
        let mut state = state.borrow_mut();

        let cover = state.covers.get_mut(&cover_id).ok_or("Cover not found")?;
        let old_capacity = cover.capacity.clone();


        if cover.cover_values > maxamount {
            return Err("Wrong Pool".to_string());
        }
        
        cover.cover_name = cover_name;
        cover.risk_type = risk_type;
        cover.chains = chains;
        cover.capacity = capacity.clone();
        cover.cost = cost;
        cover.cid = cid;
        cover.capacity_amount = capacity.clone();
        
        Ok::<Nat, String>(old_capacity)
    });
    let old_cover_cap = old_capacity.unwrap();
    let difference = if old_cover_cap.clone() > capacity.clone() {
        old_cover_cap.clone() - capacity.clone()
    } else {
        capacity.clone() - old_cover_cap.clone()
    };
    
    let _: Result<(), String> = if old_cover_cap > capacity {
        call(pool_contract, "increasePercentageSplit", (pool_id.clone(), difference.clone())).await
            .map_err(|_| "Error increasing percentage split".to_string())
    } else {
        call(pool_contract, "reducePercentageSplit", (pool_id.clone(), difference.clone())).await
            .map_err(|_| "Error decreasing percentage split".to_string())
    };

    Ok(())
}

#[update(name = "purchaseCover")]
pub async fn purchase_cover(cover_id: Nat, cover_value: Nat, cover_period: Nat, cover_fee: Nat) -> Result<(), String> {
    let caller = ic_cdk::caller();
    let bqbtc_address = STATE.with(| state | {
        let state = state.borrow();
        let bqbtc_address = state.bqbtc_address.unwrap();

        bqbtc_address
    });


    let burn_result: Result<(), _> = call(bqbtc_address, "burn", (caller, cover_fee.clone())).await;
    burn_result.map_err(|e| format!("Error burning tokens: {:?}", e))?;

    let cover = STATE.with(|state| {
        let mut state = state.borrow_mut();
        let cover = state.covers.get_mut(&cover_id).ok_or("Cover not found")?;

        if cover_value > cover.max_amount {
            return Err("Insufficient capacity".to_string());
        }

        cover.cover_values += cover_value.clone();
        cover.max_amount -= cover_value.clone();
        Ok(cover.clone())
    })?;

    STATE.with(|state| {
        let mut state = state.borrow_mut();
        let user_cover_map = state.user_covers.entry(caller).or_insert_with(HashMap::new);

        user_cover_map.insert(cover_id.clone(), GenericCoverInfo {
            user: caller,
            cover_id,
            risk_type: cover.risk_type.clone(),
            cover_name: cover.cover_name.clone(),
            cover_value,
            claim_paid: Nat::from(0u64),
            cover_period: cover_period.clone(),
            end_day: Nat::from(ic_cdk::api::time() / 1_000_000_000) + cover_period * Nat::from(86400u64),
            is_active: true,
        });

        if !state.participants.contains(&caller) {
            state.participants.push(caller);
        }
        *state.participation.entry(caller).or_insert(Nat::from(0u64)) += Nat::from(1u64);

        Ok(())
    })
}

#[update(name = "updateUserCoverValue")]
pub async fn update_user_cover_value(user: Principal, cover_id: Nat, claim_paid: Nat) -> Result<(), String>{
    STATE.with(| state | {
        let mut state = state.borrow_mut();
        let user_cover = state.user_covers.get_mut(&user).ok_or("error getting cover").unwrap().get_mut(&cover_id).ok_or("error getting cover info").unwrap();
        user_cover.cover_value -= claim_paid.clone();
        user_cover.claim_paid += claim_paid;
        Ok(())
    })
}

#[update(name = "claimPayoutForLP")]
pub async fn claim_payout_for_lp(pool_id: Nat) -> Result<(), String> {
    let caller = ic_cdk::caller();
    let (lp_contract, bqbtc_address) = STATE.with(|state| {
        let lp_contract = state.borrow().lp_contract.ok_or("LP contract address not set".to_string()).unwrap();
        let bqbtc_address= state.borrow().bqbtc_address.ok_or("bqBTC canister address not set".to_string()).unwrap();

        (lp_contract, bqbtc_address)
    });

    let deposit_info_result: Result<(Deposit,), _> = call(lp_contract, "getUserDeposit", (pool_id.clone(), caller)).await;
    let (deposit_info,) = deposit_info_result.map_err(|_| "Failed to get user deposit information")?;

    if deposit_info.status != Status::Active {
        return Err("Deposit is not active".to_string());
    }

    let last_claim_time = STATE.with(|state| {
        let state = state.borrow();
        state.lp_claims.get(&caller)
            .and_then(|claims| claims.get(&pool_id).cloned())
            .unwrap_or(deposit_info.start_date.clone())
    });

    let mut current_time = Nat::from(ic_cdk::api::time() / 1_000_000_000);
    if current_time > deposit_info.expiry_date {
        current_time = deposit_info.expiry_date;
    }
    let claimable_days = (current_time.clone() - last_claim_time.clone()) / Nat::from(86400u64); 
    
    if claimable_days <= Nat::from(0u64) {
        return Err("No claimable reward".to_string());
    }

    let claimable_amount = deposit_info.daily_payout.clone() * claimable_days.clone();

    let mint_result: Result<(), _> = call(bqbtc_address, "mint", (caller, claimable_amount.clone())).await;
    mint_result.map_err(|_| "Error minting BQ BTC".to_string())?;

    STATE.with(|state| {
        let mut state = state.borrow_mut();
        state.lp_claims.entry(caller)
            .or_insert_with(HashMap::new)
            .insert(pool_id, current_time);
    });

    Ok(())
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

#[update(name = "deleteExpiredUserCovers")]
pub async fn delete_expired_user_covers(user: Principal) -> Result<(), String> {
    STATE.with(|state| {
        let mut state = state.borrow_mut();
        let cover_ids = state.cover_ids.clone();
        let user_covers = state.user_covers.get_mut(&user).ok_or("Error getting user covers")?;
        
        let current_time = Nat::from(ic_cdk::api::time() / 1_000_000_000);

        let expired_ids: Vec<Nat> = cover_ids
            .iter()
            .filter_map(|id| {
                if let Some(user_cover) = user_covers.get_mut(id) {
                    if user_cover.is_active && current_time > user_cover.end_day {
                        user_cover.is_active = false;
                        return Some(id.clone());
                    }
                }
                None
            })
            .collect();

        for id in expired_ids {
            user_covers.remove(&id);
        }

        Ok(())
    })
}

#[update(name = "updateMaxAmount")]
pub async fn update_max_amount(cover_id: Nat) -> Result<(), String> {
    let caller = ic_cdk::caller();
    let (cover, pool_contract) = STATE.with(| state | {
        let state = state.borrow();
        let pool_contract = state.lp_contract.unwrap();
        let cover = state.covers.get(&cover_id).ok_or("error getting cover").unwrap();
        (cover.clone(), pool_contract)
    });

    if cover.capacity <= Nat::from(0u64) {
        return Err("Invalid cover capacity".to_string());
    }

    let pool_details: Result<(Pool,), _> = call(pool_contract, "getPool", (caller, cover.pool_id.clone())).await;
    let (pool, ) = pool_details.map_err(|_| "Failed to get pool")?;
    let precision = Nat::from(1_000_000_000_000_000_000u128);
    let amount = (pool.tvl * (cover.capacity * precision.clone() / Nat::from(100u64))) / precision;

    STATE.with(| state | {
        let mut state = state.borrow_mut();
        let cover = state.covers.get_mut(&cover_id).ok_or("error getting cover").unwrap();
        cover.capacity_amount = amount.clone();
        cover.max_amount = amount - cover.cover_values.clone();
    });

    Ok(())
}

#[query(name = "getAllUserCovers")]
pub async fn get_all_user_covers(user: Principal) -> Result<Vec<GenericCoverInfo>, String> {
    STATE.with(| state | {
        let state = state.borrow();
        let user_covers = state.user_covers.get(&user).ok_or_else(|| "User has no covers".to_string())?;
        let covers: Vec<GenericCoverInfo> = user_covers
            .values()
            .filter(|user_cover| user_cover.cover_value > Nat::from(0u64))
            .cloned()
            .collect();

        Ok(covers)
    })
}

#[query(name = "getAllAvailableCovers")]
pub async fn get_all_available_covers() -> Result<Vec<Cover>, String> {
    STATE.with(| state | {
        let state = state.borrow();
        let available_covers = state.covers.values().cloned().collect();

        Ok(available_covers)
    })
}

#[query(name = "getCoverInfo")]
pub async fn get_cover_info(cover_id: Nat) -> Result<Cover, String> {
    STATE.with(| state | {
        let state = state.borrow();
        let cover = state.covers.get(&cover_id).ok_or_else(|| "cover doesnt exist")?;

        Ok(cover.clone())
    })
}

#[query(name = "getUserCoverInfo")]
pub async fn get_user_cover_info(user: Principal, cover_id: Nat) -> Result<GenericCoverInfo, String> {
    STATE.with(| state | {
        let state = state.borrow();
        let cover = state.user_covers.get(&user).ok_or_else(|| "cover doesnt exist")?.get(&cover_id).ok_or_else(|| "user doesnt have this cover")?;

        Ok(cover.clone())
    })
}

#[query(name = "getDepositClaimableDays")]
pub async fn get_deposit_claimable_days(user: Principal, pool_id: Nat) -> Result<Nat, String> {
    let lp_contract = STATE.with(| state | {
        let state = state.borrow();
        let lp_contract = state.lp_contract.ok_or_else(|| "error getting pool canister id").unwrap();

        lp_contract
    });

    let deposit_info_result: Result<(Deposit,), _> = call(lp_contract, "getUserDeposit", (pool_id.clone(), user)).await;
    let (deposit_info,) = deposit_info_result.map_err(|_| "Failed to get user deposit information")?;
    
    let last_claim_time = STATE.with(|state| {
        let state = state.borrow();
        state.lp_claims.get(&user)
            .and_then(|claims| claims.get(&pool_id).cloned())
            .unwrap_or(deposit_info.start_date.clone())
    });

    let mut current_time = Nat::from(ic_cdk::api::time() / 1_000_000_000);
    if current_time > deposit_info.expiry_date {
        current_time = deposit_info.expiry_date;
    }
    let claimable_days = (current_time.clone() - last_claim_time.clone()) / Nat::from(86400u64); 

    Ok(claimable_days)
}

#[query(name = "getLastClaimTime")]
pub async fn get_last_claim_time(user: Principal, pool_id: Nat) -> Result<Nat, String> {
    STATE.with(| state | {
        let state = state.borrow();
        let last_claim = state.lp_claims.get(&user).ok_or_else(|| "user doesnt have claim yet")?.get(&pool_id).ok_or_else(|| "no claim made yet")?;

        Ok(last_claim.clone())
    })
}

ic_cdk::export_candid!();