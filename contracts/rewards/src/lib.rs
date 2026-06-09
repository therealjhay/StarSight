#![no_std]

use soroban_sdk::{contract, contracterror, contractimpl, contracttype, token, Address, Env};

/// Cross-contract interface for the reputation contract (no direct lib import).
mod rep_contract {
    use soroban_sdk::{contractclient, Address, Env};

    #[contractclient(name = "ReputationIfaceClient")]
    pub trait ReputationInterface {
        fn get_accuracy(env: Env, agent: Address) -> u32;
    }
}

/// Contract that distributes XLM rewards to agents based on reputation scores.
#[contract]
pub struct Rewards;

/// Typed storage keys for the rewards contract.
#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Admin,
    ReputationContract,
    TotalDistributed,
    AgentClaimed(Address),
    RewardPool,
    /// Stellar Asset Contract address for XLM (required for token transfers).
    XlmToken,
}

/// Typed errors for the rewards contract.
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    NotInitialized = 1,
    AlreadyInitialized = 2,
    Unauthorized = 3,
    InsufficientAccuracy = 4,
    InsufficientPool = 5,
    ZeroReward = 6,
    XlmTokenNotSet = 7,
}

/// Minimum accuracy (50%) required to claim rewards, in basis points.
const MIN_ACCURACY_BPS: u32 = 5_000;

fn require_initialized(env: &Env) -> Result<(), Error> {
    if !env.storage().instance().has(&DataKey::Admin) {
        return Err(Error::NotInitialized);
    }
    Ok(())
}

fn require_admin(env: &Env) -> Result<Address, Error> {
    let admin: Address = env
        .storage()
        .instance()
        .get(&DataKey::Admin)
        .ok_or(Error::NotInitialized)?;
    admin.require_auth();
    Ok(admin)
}

fn xlm_client(env: &Env) -> Result<token::Client<'_>, Error> {
    let xlm: Address = env
        .storage()
        .instance()
        .get(&DataKey::XlmToken)
        .ok_or(Error::XlmTokenNotSet)?;
    Ok(token::Client::new(env, &xlm))
}

#[contractimpl]
impl Rewards {
    /// Initializes the rewards contract with admin and reputation contract addresses.
    ///
    /// # Arguments
    /// * `admin` — address with pool administration rights.
    /// * `reputation_contract` — deployed reputation contract used for accuracy lookups.
    ///
    /// # Panics
    /// Panics (via `AlreadyInitialized`) if already initialized.
    pub fn initialize(env: Env, admin: Address, reputation_contract: Address) -> Result<(), Error> {
        if env.storage().instance().has(&DataKey::Admin) {
            return Err(Error::AlreadyInitialized);
        }
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage()
            .instance()
            .set(&DataKey::ReputationContract, &reputation_contract);
        env.storage().instance().set(&DataKey::RewardPool, &0i128);
        env.storage().instance().set(&DataKey::TotalDistributed, &0i128);
        Ok(())
    }

    /// Sets the XLM Stellar Asset Contract address. Admin only.
    ///
    /// Must be called once after `initialize` before `deposit` or `claim_reward`
    /// can transfer tokens.
    ///
    /// # Arguments
    /// * `xlm_token` — SAC contract address for native XLM.
    ///
    /// # Panics
    /// Panics on `NotInitialized` or `Unauthorized`.
    pub fn set_xlm_token(env: Env, xlm_token: Address) -> Result<(), Error> {
        require_admin(&env)?;
        env.storage().instance().set(&DataKey::XlmToken, &xlm_token);
        Ok(())
    }

    /// Deposits XLM into the reward pool.
    ///
    /// # Arguments
    /// * `from` — signing address funding the pool.
    /// * `amount` — XLM amount in stroops (7 decimal places).
    ///
    /// # Panics
    /// Panics on `NotInitialized`, `Unauthorized`, `XlmTokenNotSet`, or if the
    /// token transfer fails.
    pub fn deposit(env: Env, from: Address, amount: i128) -> Result<(), Error> {
        require_initialized(&env)?;

        from.require_auth();

        if amount <= 0 {
            return Err(Error::InsufficientPool);
        }

        let token = xlm_client(&env)?;
        let contract = env.current_contract_address();
        token.transfer(&from, &contract, &amount);

        let mut pool: i128 = env
            .storage()
            .instance()
            .get(&DataKey::RewardPool)
            .unwrap_or(0);
        pool = pool
            .checked_add(amount)
            .expect("reward pool overflow");
        env.storage().instance().set(&DataKey::RewardPool, &pool);
        Ok(())
    }

    /// Claims XLM rewards for an agent based on their on-chain accuracy score.
    ///
    /// # Arguments
    /// * `agent` — signing agent address claiming rewards.
    ///
    /// Reward formula: `(accuracy_bps / 10000) * pool_share_per_agent` where
    /// `pool_share_per_agent` is the current `RewardPool` balance.
    /// Agents with `accuracy_bps < 5000` (50%) receive zero.
    ///
    /// # Panics
    /// Panics on `NotInitialized`, `Unauthorized`, `InsufficientAccuracy`,
    /// `ZeroReward`, `InsufficientPool`, or `XlmTokenNotSet`.
    pub fn claim_reward(env: Env, agent: Address) -> Result<i128, Error> {
        require_initialized(&env)?;

        agent.require_auth();

        let reputation_addr: Address = env
            .storage()
            .instance()
            .get(&DataKey::ReputationContract)
            .ok_or(Error::NotInitialized)?;

        let reputation_client = rep_contract::ReputationIfaceClient::new(&env, &reputation_addr);
        let accuracy_bps = reputation_client.get_accuracy(&agent);

        if accuracy_bps < MIN_ACCURACY_BPS {
            return Err(Error::InsufficientAccuracy);
        }

        let pool: i128 = env
            .storage()
            .instance()
            .get(&DataKey::RewardPool)
            .unwrap_or(0);

        if pool <= 0 {
            return Err(Error::InsufficientPool);
        }

        let pool_share_per_agent = pool;
        let reward = (accuracy_bps as i128)
            .checked_mul(pool_share_per_agent)
            .expect("reward mul overflow")
            / 10_000;

        if reward <= 0 {
            return Err(Error::ZeroReward);
        }

        let reward = reward.min(pool);

        let token = xlm_client(&env)?;
        let contract = env.current_contract_address();
        token.transfer(&contract, &agent, &reward);

        let new_pool = pool - reward;
        env.storage().instance().set(&DataKey::RewardPool, &new_pool);

        let claimed: i128 = env
            .storage()
            .instance()
            .get(&DataKey::AgentClaimed(agent.clone()))
            .unwrap_or(0);
        let new_claimed = claimed
            .checked_add(reward)
            .expect("claimed overflow");
        env.storage()
            .instance()
            .set(&DataKey::AgentClaimed(agent), &new_claimed);

        let total: i128 = env
            .storage()
            .instance()
            .get(&DataKey::TotalDistributed)
            .unwrap_or(0);
        let new_total = total
            .checked_add(reward)
            .expect("total distributed overflow");
        env.storage()
            .instance()
            .set(&DataKey::TotalDistributed, &new_total);

        Ok(reward)
    }

    /// Returns the current reward pool balance (accounted XLM).
    ///
    /// # Panics
    /// Panics on `NotInitialized`.
    pub fn get_pool_balance(env: Env) -> Result<i128, Error> {
        require_initialized(&env)?;

        Ok(env
            .storage()
            .instance()
            .get(&DataKey::RewardPool)
            .unwrap_or(0))
    }

    /// Returns the cumulative XLM claimed by an agent.
    ///
    /// # Arguments
    /// * `agent` — agent address to query.
    ///
    /// # Panics
    /// Panics on `NotInitialized`.
    pub fn get_agent_claimed(env: Env, agent: Address) -> Result<i128, Error> {
        require_initialized(&env)?;

        Ok(env
            .storage()
            .instance()
            .get(&DataKey::AgentClaimed(agent))
            .unwrap_or(0))
    }

    /// Returns the total XLM distributed across all agents.
    ///
    /// # Panics
    /// Panics on `NotInitialized`.
    pub fn total_distributed(env: Env) -> Result<i128, Error> {
        require_initialized(&env)?;

        Ok(env
            .storage()
            .instance()
            .get(&DataKey::TotalDistributed)
            .unwrap_or(0))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use reputation::{Reputation, ReputationClient as ReputationContractClient};
    use soroban_sdk::{
        testutils::{Address as _, Ledger as _, StellarAssetContract as _},
        token::{Client as TokenClient, StellarAssetClient},
        Address, Env,
    };

    fn setup_reputation(env: &Env, agent: &Address) -> Address {
        let admin = Address::generate(env);
        let market = Address::generate(env);
        let rep_id = env.register(Reputation, ());
        let rep_client = ReputationContractClient::new(env, &rep_id);
        rep_client.initialize(&admin, &market);

        env.mock_all_auths();
        // Score two correct predictions for 100% accuracy
        rep_client.score_prediction(agent, &1_000_000, &1_000_000, &1);
        rep_client.score_prediction(agent, &1_000_000, &1_000_000, &2);
        rep_id
    }

    fn setup_xlm(env: &Env, funder: &Address) -> (Address, TokenClient<'static>) {
        let sac = env.register_stellar_asset_contract_v2(funder.clone());
        let token_addr = sac.address();
        let stellar = StellarAssetClient::new(env, &token_addr);
        stellar.mint(funder, &10_000_000_000);
        let client = TokenClient::new(env, &token_addr);
        (token_addr, client)
    }

    fn setup_rewards(
        env: &Env,
        reputation_id: &Address,
    ) -> (Address, Address, RewardsClient<'static>) {
        let admin = Address::generate(env);
        let rewards_id = env.register(Rewards, ());
        let client = RewardsClient::new(env, &rewards_id);
        client.initialize(&admin, reputation_id);
        (admin, rewards_id, client)
    }

    // --- initialize ---

    #[test]
    fn initialize_happy_path() {
        let env = Env::default();
        env.mock_all_auths();
        let admin = Address::generate(&env);
        let reputation = Address::generate(&env);
        let rewards_id = env.register(Rewards, ());
        let client = RewardsClient::new(&env, &rewards_id);

        client.initialize(&admin, &reputation);
        assert_eq!(client.get_pool_balance(), 0);
        assert_eq!(client.total_distributed(), 0);
    }

    #[test]
    fn initialize_fails_when_already_initialized() {
        let env = Env::default();
        env.mock_all_auths();
        let admin = Address::generate(&env);
        let reputation = Address::generate(&env);
        let (_a, _id, client) = setup_rewards(&env, &reputation);

        let result = client.try_initialize(&admin, &reputation);
        assert_eq!(result, Err(Ok(Error::AlreadyInitialized)));
    }

    #[test]
    fn initialize_starts_with_zero_pool() {
        let env = Env::default();
        env.mock_all_auths();
        let reputation = Address::generate(&env);
        let (_a, _id, client) = setup_rewards(&env, &reputation);
        assert_eq!(client.get_pool_balance(), 0);
    }

    // --- set_xlm_token ---

    #[test]
    fn set_xlm_token_happy_path() {
        let env = Env::default();
        env.mock_all_auths();
        let funder = Address::generate(&env);
        let (xlm, _) = setup_xlm(&env, &funder);
        let reputation = Address::generate(&env);
        let (admin, _id, client) = setup_rewards(&env, &reputation);

        client.set_xlm_token(&xlm);
        let _ = admin;
    }

    #[test]
    fn set_xlm_token_fails_when_unauthorized() {
        let env = Env::default();
        let xlm = Address::generate(&env);
        let reputation = Address::generate(&env);
        let (_a, _id, client) = setup_rewards(&env, &reputation);

        let result = client.try_set_xlm_token(&xlm);
        assert!(matches!(result, Err(Err(_))));
    }

    // --- deposit ---

    #[test]
    fn deposit_happy_path() {
        let env = Env::default();
        env.mock_all_auths();
        let funder = Address::generate(&env);
        let (xlm, _) = setup_xlm(&env, &funder);
        let agent = Address::generate(&env);
        let reputation = setup_reputation(&env, &agent);
        let (admin, _id, client) = setup_rewards(&env, &reputation);
        client.set_xlm_token(&xlm);

        client.deposit(&funder, &1_000_000_000);
        assert_eq!(client.get_pool_balance(), 1_000_000_000);
        let _ = admin;
    }

    #[test]
    fn deposit_fails_when_xlm_not_set() {
        let env = Env::default();
        env.mock_all_auths();
        let funder = Address::generate(&env);
        let agent = Address::generate(&env);
        let reputation = setup_reputation(&env, &agent);
        let (_a, _id, client) = setup_rewards(&env, &reputation);

        let result = client.try_deposit(&funder, &1_000);
        assert_eq!(result, Err(Ok(Error::XlmTokenNotSet)));
    }

    #[test]
    fn deposit_fails_when_unauthorized() {
        let env = Env::default();
        env.mock_all_auths();
        let funder = Address::generate(&env);
        let (xlm, _) = setup_xlm(&env, &funder);
        let reputation = Address::generate(&env);
        let (_a, _id, client) = setup_rewards(&env, &reputation);
        client.set_xlm_token(&xlm);
        env.set_auths(&[]);

        let result = client.try_deposit(&funder, &1_000);
        assert!(matches!(result, Err(Err(_))));
    }

    #[test]
    fn deposit_fails_on_zero_amount() {
        let env = Env::default();
        env.mock_all_auths();
        let funder = Address::generate(&env);
        let (xlm, _) = setup_xlm(&env, &funder);
        let reputation = Address::generate(&env);
        let (_a, _id, client) = setup_rewards(&env, &reputation);
        client.set_xlm_token(&xlm);

        let result = client.try_deposit(&funder, &0);
        assert_eq!(result, Err(Ok(Error::InsufficientPool)));
    }

    // --- claim_reward ---

    #[test]
    fn claim_reward_happy_path() {
        let env = Env::default();
        env.mock_all_auths();
        let funder = Address::generate(&env);
        let (xlm, token) = setup_xlm(&env, &funder);
        let agent = Address::generate(&env);
        let reputation = setup_reputation(&env, &agent);
        let (_a, rewards_id, client) = setup_rewards(&env, &reputation);
        client.set_xlm_token(&xlm);
        client.deposit(&funder, &10_000_000_000);

        let reward = client.claim_reward(&agent);
        assert_eq!(reward, 10_000_000_000);
        assert_eq!(client.get_agent_claimed(&agent), 10_000_000_000);
        assert_eq!(client.total_distributed(), 10_000_000_000);
        assert_eq!(client.get_pool_balance(), 0);
        assert_eq!(token.balance(&agent), 10_000_000_000);
        assert_eq!(token.balance(&rewards_id), 0);
    }

    #[test]
    fn claim_reward_fails_insufficient_accuracy() {
        let env = Env::default();
        env.mock_all_auths();
        let funder = Address::generate(&env);
        let (xlm, _) = setup_xlm(&env, &funder);
        let agent = Address::generate(&env);

        let rep_admin = Address::generate(&env);
        let market = Address::generate(&env);
        let rep_id = env.register(Reputation, ());
        let rep_client = ReputationContractClient::new(&env, &rep_id);
        rep_client.initialize(&rep_admin, &market);
        // One correct out of three => 33% accuracy, below the 50% threshold
        rep_client.score_prediction(&agent, &1_000_000, &1_000_000, &1);
        rep_client.score_prediction(&agent, &1_200_000, &1_000_000, &2);
        rep_client.score_prediction(&agent, &1_200_000, &1_000_000, &3);

        let (_a, _id, client) = setup_rewards(&env, &rep_id);
        client.set_xlm_token(&xlm);
        client.deposit(&funder, &1_000_000_000);

        let result = client.try_claim_reward(&agent);
        assert_eq!(result, Err(Ok(Error::InsufficientAccuracy)));
    }

    #[test]
    fn claim_reward_fails_when_unauthorized() {
        let env = Env::default();
        env.mock_all_auths();
        let funder = Address::generate(&env);
        let (xlm, _) = setup_xlm(&env, &funder);
        let agent = Address::generate(&env);
        let reputation = setup_reputation(&env, &agent);
        let (_a, _id, client) = setup_rewards(&env, &reputation);
        client.set_xlm_token(&xlm);
        client.deposit(&funder, &1_000_000_000);
        env.set_auths(&[]);

        let result = client.try_claim_reward(&agent);
        assert!(matches!(result, Err(Err(_))));
    }

    #[test]
    fn claim_reward_fails_on_empty_pool() {
        let env = Env::default();
        env.mock_all_auths();
        let agent = Address::generate(&env);
        let reputation = setup_reputation(&env, &agent);
        let funder = Address::generate(&env);
        let (xlm, _) = setup_xlm(&env, &funder);
        let (_a, _id, client) = setup_rewards(&env, &reputation);
        client.set_xlm_token(&xlm);

        let result = client.try_claim_reward(&agent);
        assert_eq!(result, Err(Ok(Error::InsufficientPool)));
    }

    // --- get_pool_balance ---

    #[test]
    fn get_pool_balance_happy_path() {
        let env = Env::default();
        env.mock_all_auths();
        let funder = Address::generate(&env);
        let (xlm, _) = setup_xlm(&env, &funder);
        let reputation = Address::generate(&env);
        let (_a, _id, client) = setup_rewards(&env, &reputation);
        client.set_xlm_token(&xlm);
        client.deposit(&funder, &500);

        assert_eq!(client.get_pool_balance(), 500);
    }

    #[test]
    fn get_pool_balance_fails_when_not_initialized() {
        let env = Env::default();
        let rewards_id = env.register(Rewards, ());
        let client = RewardsClient::new(&env, &rewards_id);

        let result = client.try_get_pool_balance();
        assert_eq!(result, Err(Ok(Error::NotInitialized)));
    }

    // --- get_agent_claimed ---

    #[test]
    fn get_agent_claimed_happy_path() {
        let env = Env::default();
        env.mock_all_auths();
        let funder = Address::generate(&env);
        let (xlm, _) = setup_xlm(&env, &funder);
        let agent = Address::generate(&env);
        let reputation = setup_reputation(&env, &agent);
        let (_a, _id, client) = setup_rewards(&env, &reputation);
        client.set_xlm_token(&xlm);
        client.deposit(&funder, &2_000);
        client.claim_reward(&agent);

        assert_eq!(client.get_agent_claimed(&agent), 2_000);
    }

    #[test]
    fn get_agent_claimed_zero_by_default() {
        let env = Env::default();
        env.mock_all_auths();
        let reputation = Address::generate(&env);
        let (_a, _id, client) = setup_rewards(&env, &reputation);
        let agent = Address::generate(&env);

        assert_eq!(client.get_agent_claimed(&agent), 0);
    }

    #[test]
    fn get_agent_claimed_fails_when_not_initialized() {
        let env = Env::default();
        let rewards_id = env.register(Rewards, ());
        let client = RewardsClient::new(&env, &rewards_id);
        let agent = Address::generate(&env);

        let result = client.try_get_agent_claimed(&agent);
        assert_eq!(result, Err(Ok(Error::NotInitialized)));
    }

    // --- total_distributed ---

    #[test]
    fn total_distributed_happy_path() {
        let env = Env::default();
        env.mock_all_auths();
        let funder = Address::generate(&env);
        let (xlm, _) = setup_xlm(&env, &funder);
        let agent = Address::generate(&env);
        let reputation = setup_reputation(&env, &agent);
        let (_a, _id, client) = setup_rewards(&env, &reputation);
        client.set_xlm_token(&xlm);
        client.deposit(&funder, &3_000);
        client.claim_reward(&agent);

        assert_eq!(client.total_distributed(), 3_000);
    }

    #[test]
    fn total_distributed_starts_at_zero() {
        let env = Env::default();
        env.mock_all_auths();
        let reputation = Address::generate(&env);
        let (_a, _id, client) = setup_rewards(&env, &reputation);

        assert_eq!(client.total_distributed(), 0);
    }

    #[test]
    fn total_distributed_fails_when_not_initialized() {
        let env = Env::default();
        let rewards_id = env.register(Rewards, ());
        let client = RewardsClient::new(&env, &rewards_id);

        let result = client.try_total_distributed();
        assert_eq!(result, Err(Ok(Error::NotInitialized)));
    }
}
