#![no_std]

use soroban_sdk::{contract, contracterror, contractimpl, contracttype, Address, Env};

/// Contract that scores agent prediction accuracy over time.
#[contract]
pub struct Reputation;

/// Typed storage keys for the reputation contract.
#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    AgentScore(Address),
    Admin,
    PredictionMarket,
}

/// Cumulative reputation metrics for a single agent.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReputationScore {
    pub agent: Address,
    pub total_predictions: u32,
    pub correct_predictions: u32,
    pub accuracy_bps: u32,
    pub streak: u32,
    pub last_scored_at: u64,
}

/// Typed errors for the reputation contract.
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    NotInitialized = 1,
    AlreadyInitialized = 2,
    Unauthorized = 3,
    AgentNotFound = 4,
}

const TOLERANCE_BPS: i128 = 500; // 5% of 10000 basis points

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

/// Returns true when `|predicted - actual| / actual <= 0.05`.
fn is_prediction_correct(predicted: i128, actual: i128) -> bool {
    if actual == 0 {
        return predicted == 0;
    }
    let diff = if predicted > actual {
        predicted - actual
    } else {
        actual - predicted
    };
    let actual_abs = if actual < 0 { -actual } else { actual };
    diff.saturating_mul(10_000) <= actual_abs.saturating_mul(TOLERANCE_BPS)
}

fn default_score(_env: &Env, agent: Address) -> ReputationScore {
    ReputationScore {
        agent,
        total_predictions: 0,
        correct_predictions: 0,
        accuracy_bps: 0,
        streak: 0,
        last_scored_at: 0,
    }
}

fn update_accuracy_bps(score: &mut ReputationScore) {
    if score.total_predictions == 0 {
        score.accuracy_bps = 0;
    } else {
        score.accuracy_bps = ((score.correct_predictions as u64) * 10_000
            / score.total_predictions as u64) as u32;
    }
}

#[contractimpl]
impl Reputation {
    /// Initializes the contract with an admin and prediction market address.
    ///
    /// # Arguments
    /// * `admin` — address authorized to score predictions.
    /// * `prediction_market` — deployed prediction market contract.
    ///
    /// # Panics
    /// Panics (via `AlreadyInitialized`) if already initialized.
    pub fn initialize(
        env: Env,
        admin: Address,
        prediction_market: Address,
    ) -> Result<(), Error> {
        if env.storage().instance().has(&DataKey::Admin) {
            return Err(Error::AlreadyInitialized);
        }
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage()
            .instance()
            .set(&DataKey::PredictionMarket, &prediction_market);
        Ok(())
    }

    /// Scores a resolved prediction and updates the agent's reputation atomically.
    ///
    /// # Arguments
    /// * `agent` — agent who made the prediction.
    /// * `predicted_value` — agent's forecast scaled by 1_000_000.
    /// * `actual_value` — observed value scaled by 1_000_000.
    /// * `prediction_id` — ID of the scored prediction (stored for audit trail).
    ///
    /// A prediction is correct when `|predicted - actual| / actual <= 0.05`.
    ///
    /// # Panics
    /// Panics on `NotInitialized` or `Unauthorized`.
    pub fn score_prediction(
        env: Env,
        agent: Address,
        predicted_value: i128,
        actual_value: i128,
        prediction_id: u64,
    ) -> Result<(), Error> {
        require_admin(&env)?;
        let _ = prediction_id; // recorded in invocation events; reserved for future dedup

        let key = DataKey::AgentScore(agent.clone());
        let mut score: ReputationScore = env
            .storage()
            .instance()
            .get(&key)
            .unwrap_or_else(|| default_score(&env, agent.clone()));

        score.total_predictions = score
            .total_predictions
            .checked_add(1)
            .expect("total_predictions overflow");

        let correct = is_prediction_correct(predicted_value, actual_value);
        if correct {
            score.correct_predictions = score
                .correct_predictions
                .checked_add(1)
                .expect("correct_predictions overflow");
            score.streak = score.streak.checked_add(1).expect("streak overflow");
        } else {
            score.streak = 0;
        }

        update_accuracy_bps(&mut score);
        score.last_scored_at = env.ledger().timestamp();

        env.storage().instance().set(&key, &score);
        Ok(())
    }

    /// Returns the full reputation score for an agent.
    ///
    /// # Arguments
    /// * `agent` — agent address to query.
    ///
    /// # Panics
    /// Panics on `NotInitialized`. Returns a zeroed score for agents not yet scored.
    pub fn get_score(env: Env, agent: Address) -> Result<ReputationScore, Error> {
        require_initialized(&env)?;

        Ok(env
            .storage()
            .instance()
            .get(&DataKey::AgentScore(agent.clone()))
            .unwrap_or_else(|| default_score(&env, agent)))
    }

    /// Returns the agent's accuracy in basis points (0–10000).
    ///
    /// # Arguments
    /// * `agent` — agent address to query.
    ///
    /// # Panics
    /// Panics on `NotInitialized`.
    pub fn get_accuracy(env: Env, agent: Address) -> Result<u32, Error> {
        let score = Self::get_score(env, agent)?;
        Ok(score.accuracy_bps)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use soroban_sdk::{testutils::Address as _, Env};

    fn setup(env: &Env) -> (Address, Address, ReputationClient<'static>) {
        let admin = Address::generate(env);
        let market = Address::generate(env);
        let contract_id = env.register(Reputation, ());
        let client = ReputationClient::new(env, &contract_id);
        client.initialize(&admin, &market);
        (admin, market, client)
    }

    // --- initialize ---

    #[test]
    fn initialize_happy_path() {
        let env = Env::default();
        env.mock_all_auths();
        let admin = Address::generate(&env);
        let market = Address::generate(&env);
        let contract_id = env.register(Reputation, ());
        let client = ReputationClient::new(&env, &contract_id);

        client.initialize(&admin, &market);
        let agent = Address::generate(&env);
        assert_eq!(client.get_accuracy(&agent), 0);
    }

    #[test]
    fn initialize_fails_when_already_initialized() {
        let env = Env::default();
        env.mock_all_auths();
        let (admin, market, client) = setup(&env);

        let result = client.try_initialize(&admin, &market);
        assert_eq!(result, Err(Ok(Error::AlreadyInitialized)));
    }

    #[test]
    fn initialize_allows_scoring_after_setup() {
        let env = Env::default();
        env.mock_all_auths();
        let (_admin, _market, client) = setup(&env);
        let agent = Address::generate(&env);

        client.score_prediction(&agent, &1_000_000, &1_000_000, &1);
        assert_eq!(client.get_accuracy(&agent), 10_000);
    }

    // --- score_prediction ---

    #[test]
    fn score_prediction_correct_within_tolerance() {
        let env = Env::default();
        env.mock_all_auths();
        let (_admin, _market, client) = setup(&env);
        let agent = Address::generate(&env);

        // 4% error: predicted 1_040_000 vs actual 1_000_000
        client.score_prediction(&agent, &1_040_000, &1_000_000, &1);
        let score = client.get_score(&agent);
        assert_eq!(score.correct_predictions, 1);
        assert_eq!(score.accuracy_bps, 10_000);
        assert_eq!(score.streak, 1);
    }

    #[test]
    fn score_prediction_incorrect_beyond_tolerance() {
        let env = Env::default();
        env.mock_all_auths();
        let (_admin, _market, client) = setup(&env);
        let agent = Address::generate(&env);

        // 6% error
        client.score_prediction(&agent, &1_060_000, &1_000_000, &1);
        let score = client.get_score(&agent);
        assert_eq!(score.correct_predictions, 0);
        assert_eq!(score.accuracy_bps, 0);
        assert_eq!(score.streak, 0);
    }

    #[test]
    fn score_prediction_fails_when_unauthorized() {
        let env = Env::default();
        let (_admin, _market, client) = setup(&env);
        let agent = Address::generate(&env);

        let result = client.try_score_prediction(&agent, &1_000_000, &1_000_000, &1);
        assert!(matches!(result, Err(Err(_))));
    }

    #[test]
    fn score_prediction_updates_accuracy_over_multiple() {
        let env = Env::default();
        env.mock_all_auths();
        let (_admin, _market, client) = setup(&env);
        let agent = Address::generate(&env);

        client.score_prediction(&agent, &1_000_000, &1_000_000, &1);
        client.score_prediction(&agent, &1_060_000, &1_000_000, &2);
        assert_eq!(client.get_accuracy(&agent), 5_000);
        let score = client.get_score(&agent);
        assert_eq!(score.streak, 0);
    }

    // --- get_score ---

    #[test]
    fn get_score_happy_path() {
        let env = Env::default();
        env.mock_all_auths();
        let (_admin, _market, client) = setup(&env);
        let agent = Address::generate(&env);

        client.score_prediction(&agent, &1_000_000, &1_000_000, &1);
        let score = client.get_score(&agent);
        assert_eq!(score.agent, agent);
        assert_eq!(score.total_predictions, 1);
    }

    #[test]
    fn get_score_returns_default_for_new_agent() {
        let env = Env::default();
        env.mock_all_auths();
        let (_admin, _market, client) = setup(&env);
        let agent = Address::generate(&env);

        let score = client.get_score(&agent);
        assert_eq!(score.total_predictions, 0);
        assert_eq!(score.accuracy_bps, 0);
    }

    #[test]
    fn get_score_fails_when_not_initialized() {
        let env = Env::default();
        let contract_id = env.register(Reputation, ());
        let client = ReputationClient::new(&env, &contract_id);
        let agent = Address::generate(&env);

        let result = client.try_get_score(&agent);
        assert_eq!(result, Err(Ok(Error::NotInitialized)));
    }

    // --- get_accuracy ---

    #[test]
    fn get_accuracy_happy_path() {
        let env = Env::default();
        env.mock_all_auths();
        let (_admin, _market, client) = setup(&env);
        let agent = Address::generate(&env);

        client.score_prediction(&agent, &1_000_000, &1_000_000, &1);
        client.score_prediction(&agent, &1_000_000, &1_000_000, &2);
        assert_eq!(client.get_accuracy(&agent), 10_000);
    }

    #[test]
    fn get_accuracy_zero_for_unscored_agent() {
        let env = Env::default();
        env.mock_all_auths();
        let (_admin, _market, client) = setup(&env);
        let agent = Address::generate(&env);

        assert_eq!(client.get_accuracy(&agent), 0);
    }

    #[test]
    fn get_accuracy_fails_when_not_initialized() {
        let env = Env::default();
        let contract_id = env.register(Reputation, ());
        let client = ReputationClient::new(&env, &contract_id);
        let agent = Address::generate(&env);

        let result = client.try_get_accuracy(&agent);
        assert_eq!(result, Err(Ok(Error::NotInitialized)));
    }
}
