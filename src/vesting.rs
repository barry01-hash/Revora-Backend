use soroban_sdk::{contract, contractimpl, contracttype, Address, Env, Map, Vec, symbol_short};

const SCHEMA_VERSION: &str = "1.0";

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VestingCreatedEvent {
    pub beneficiary: Address,
    pub total_amount: i128,
    pub start_time: u64,
    pub cliff_time: u64,
    pub end_time: u64,
    pub timestamp: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PartialClaimEvent {
    pub beneficiary: Address,
    pub amount: i128,
    pub timestamp: u64,
    pub total_claimed: i128,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VestingSchedule {
    pub total_amount: i128,
    pub start_time: u64,
    pub cliff_time: u64,
    pub end_time: u64,
    pub claimed: i128,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PartialClaim {
    pub amount: i128,
    pub timestamp: u64,
}

#[contract]
pub struct VestingContract;

#[contractimpl]
impl VestingContract {
    pub fn initialize(env: Env, admin: Address) {
        env.storage().instance().set(&"admin", &admin);
    }

    pub fn create_vesting(
        env: Env,
        beneficiary: Address,
        total_amount: i128,
        start_time: u64,
        cliff_time: u64,
        end_time: u64,
    ) {
        // Validate times
        assert!(start_time <= cliff_time && cliff_time <= end_time, "Invalid vesting times");

        let schedule = VestingSchedule {
            total_amount,
            start_time,
            cliff_time,
            end_time,
            claimed: 0,
        };

        env.storage().persistent().set(&beneficiary, &schedule);

        // Emit event
        let event = VestingCreatedEvent {
            beneficiary: beneficiary.clone(),
            total_amount,
            start_time,
            cliff_time,
            end_time,
            timestamp: env.ledger().timestamp(),
        };
        env.events().publish(("vesting", symbol_short!("created")), event);
    }

    pub fn claim(env: Env, beneficiary: Address, amount: i128) {
        beneficiary.require_auth();

        let mut schedule: VestingSchedule = env.storage().persistent().get(&beneficiary).unwrap();

        let current_time = env.ledger().timestamp();
        let vested = Self::calculate_vested(&schedule, current_time);

        assert!(vested >= schedule.claimed + amount, "Claim amount exceeds vested amount");
        assert!(amount > 0, "Claim amount must be positive");

        schedule.claimed += amount;

        // Record partial claim
        let claim = PartialClaim {
            amount,
            timestamp: current_time,
        };

        let mut claims: Vec<PartialClaim> = env.storage().persistent().get(&(&beneficiary, "claims")).unwrap_or(Vec::new(&env));
        claims.push_back(claim);

        env.storage().persistent().set(&(&beneficiary, "claims"), &claims);
        env.storage().persistent().set(&beneficiary, &schedule);

        // Emit event
        let event = PartialClaimEvent {
            beneficiary: beneficiary.clone(),
            amount,
            timestamp: current_time,
            total_claimed: schedule.claimed,
        };
        env.events().publish(("vesting", symbol_short!("claimed")), event);

        // Transfer tokens (assuming token contract exists)
        // For now, just log
    }

    pub(crate) fn calculate_vested(schedule: &VestingSchedule, current_time: u64) -> i128 {
        if current_time < schedule.cliff_time {
            0
        } else if current_time >= schedule.end_time {
            schedule.total_amount
        } else {
            let elapsed = current_time - schedule.cliff_time;
            let vesting_duration = schedule.end_time - schedule.cliff_time;
            (schedule.total_amount * elapsed as i128) / vesting_duration as i128
        }
    }
}