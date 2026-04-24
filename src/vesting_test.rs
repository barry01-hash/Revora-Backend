#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::testutils::{Address as _, Ledger};
    use soroban_sdk::{Address, Env};

    #[test]
    fn test_create_vesting() {
        let env = Env::default();
        let contract_id = Address::random(&env);
        env.register_contract(&contract_id, VestingContract);

        let admin = Address::random(&env);
        VestingContract::initialize(env.clone(), admin);

        let beneficiary = Address::random(&env);
        let total_amount = 1000i128;
        let start_time = 1000u64;
        let cliff_time = 2000u64;
        let end_time = 3000u64;

        VestingContract::create_vesting(env.clone(), beneficiary.clone(), total_amount, start_time, cliff_time, end_time);

        // Check schedule is stored
        let schedule: VestingSchedule = env.storage().persistent().get(&beneficiary).unwrap();
        assert_eq!(schedule.total_amount, total_amount);
        assert_eq!(schedule.claimed, 0);
    }

    #[test]
    fn test_partial_claim() {
        let env = Env::default();
        let contract_id = Address::random(&env);
        env.register_contract(&contract_id, VestingContract);

        let admin = Address::random(&env);
        VestingContract::initialize(env.clone(), admin);

        let beneficiary = Address::random(&env);
        let total_amount = 1000i128;
        let start_time = 1000u64;
        let cliff_time = 2000u64;
        let end_time = 3000u64;

        VestingContract::create_vesting(env.clone(), beneficiary.clone(), total_amount, start_time, cliff_time, end_time);

        // Set time after cliff, halfway through vesting
        env.ledger().set_timestamp(2500);

        let vested = VestingContract::calculate_vested(&VestingSchedule {
            total_amount,
            start_time,
            cliff_time,
            end_time,
            claimed: 0,
        }, 2500);

        assert_eq!(vested, 500); // Half way

        // Claim half of vested
        VestingContract::claim(env.clone(), beneficiary.clone(), 250);

        let schedule: VestingSchedule = env.storage().persistent().get(&beneficiary).unwrap();
        assert_eq!(schedule.claimed, 250);

        let claims: Vec<PartialClaim> = env.storage().persistent().get(&(&beneficiary, "claims")).unwrap();
        assert_eq!(claims.len(), 1);
        assert_eq!(claims.get(0).unwrap().amount, 250);
    }

    #[test]
    #[should_panic(expected = "Claim amount exceeds vested amount")]
    fn test_claim_more_than_vested() {
        let env = Env::default();
        let contract_id = Address::random(&env);
        env.register_contract(&contract_id, VestingContract);

        let admin = Address::random(&env);
        VestingContract::initialize(env.clone(), admin);

        let beneficiary = Address::random(&env);
        let total_amount = 1000i128;
        let start_time = 1000u64;
        let cliff_time = 2000u64;
        let end_time = 3000u64;

        VestingContract::create_vesting(env.clone(), beneficiary.clone(), total_amount, start_time, cliff_time, end_time);

        env.ledger().set_timestamp(2500);

        // Try to claim more than vested
        VestingContract::claim(env.clone(), beneficiary.clone(), 600);
    }

    #[test]
    #[should_panic(expected = "Claim amount must be positive")]
    fn test_negative_claim() {
        let env = Env::default();
        let contract_id = Address::random(&env);
        env.register_contract(&contract_id, VestingContract);

        let admin = Address::random(&env);
        VestingContract::initialize(env.clone(), admin);

        let beneficiary = Address::random(&env);
        let total_amount = 1000i128;
        let start_time = 1000u64;
        let cliff_time = 2000u64;
        let end_time = 3000u64;

        VestingContract::create_vesting(env.clone(), beneficiary.clone(), total_amount, start_time, cliff_time, end_time);

        env.ledger().set_timestamp(2500);

        VestingContract::claim(env.clone(), beneficiary.clone(), -100);
    }
}