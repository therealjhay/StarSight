#![no_std]

use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, Address, Env, Symbol, Vec,
};

/// Contract for AI agents to post predictions and users to subscribe to agents.
#[contract]
pub struct PredictionMarket;

/// Typed storage keys for the prediction market.
#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Prediction(u64),
    PredictionCount,
    AgentSubscribers(Address),
    SubscriberAgents(Address),
    Admin,
    ReputationContract,
}

/// Kind of prediction an agent can submit.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PredictionType {
    PriceTarget,
    YieldForecast,
    RiskScore,
}

/// Lifecycle status of a prediction.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PredictionStatus {
    Pending,
    Resolved,
    Scored,
}

/// A single agent prediction on an RWA asset.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Prediction {
    pub id: u64,
    pub agent: Address,
    pub asset_id: Symbol,
    pub prediction_type: PredictionType,
    pub value: i128,
    pub confidence: u32,
    pub submitted_at: u64,
    pub resolution_ledger: u64,
    pub status: PredictionStatus,
    pub resolved_value: Option<i128>,
}

/// Typed errors for the prediction market contract.
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    NotInitialized = 1,
    AlreadyInitialized = 2,
    Unauthorized = 3,
    PredictionNotFound = 4,
    InvalidConfidence = 5,
    InvalidResolutionLedger = 6,
    PredictionNotPending = 7,
    AlreadySubscribed = 8,
    NotSubscribed = 9,
}

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

fn vec_contains_address(vec: &Vec<Address>, addr: &Address) -> bool {
    for item in vec.iter() {
        if item == *addr {
            return true;
        }
    }
    false
}

fn vec_remove_address(env: &Env, vec: &Vec<Address>, addr: &Address) -> Vec<Address> {
    let mut out = Vec::new(env);
    for item in vec.iter() {
        if item != *addr {
            out.push_back(item);
        }
    }
    out
}

#[contractimpl]
impl PredictionMarket {
    /// Initializes the contract with an admin and reputation contract address.
    ///
    /// # Arguments
    /// * `admin` — address with resolution authority.
    /// * `reputation_contract` — deployed reputation contract for scoring integration.
    ///
    /// # Panics
    /// Panics (via `AlreadyInitialized`) if already initialized.
    pub fn initialize(
        env: Env,
        admin: Address,
        reputation_contract: Address,
    ) -> Result<(), Error> {
        if env.storage().instance().has(&DataKey::Admin) {
            return Err(Error::AlreadyInitialized);
        }
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage()
            .instance()
            .set(&DataKey::ReputationContract, &reputation_contract);
        env.storage().instance().set(&DataKey::PredictionCount, &0u64);
        Ok(())
    }

    /// Submits a new prediction on behalf of an authenticated agent.
    ///
    /// # Arguments
    /// * `agent` — signing agent address.
    /// * `asset_id` — symbol of the target RWA.
    /// * `prediction_type` — kind of forecast.
    /// * `value` — predicted value scaled by 1_000_000 (6 decimal places).
    /// * `confidence` — basis points from 0 to 10000.
    /// * `resolution_ledger` — ledger sequence when the prediction may be resolved.
    ///
    /// # Returns
    /// The newly assigned prediction ID.
    ///
    /// # Panics
    /// Panics on `NotInitialized`, `Unauthorized`, `InvalidConfidence`, or
    /// `InvalidResolutionLedger`.
    pub fn submit_prediction(
        env: Env,
        agent: Address,
        asset_id: Symbol,
        prediction_type: PredictionType,
        value: i128,
        confidence: u32,
        resolution_ledger: u64,
    ) -> Result<u64, Error> {
        require_initialized(&env)?;

        agent.require_auth();

        if confidence > 10_000 {
            return Err(Error::InvalidConfidence);
        }

        let current_ledger = env.ledger().sequence() as u64;
        if resolution_ledger <= current_ledger {
            return Err(Error::InvalidResolutionLedger);
        }

        let mut count: u64 = env
            .storage()
            .instance()
            .get(&DataKey::PredictionCount)
            .unwrap_or(0);
        count = count.checked_add(1).expect("prediction count overflow");
        let prediction_id = count;

        let prediction = Prediction {
            id: prediction_id,
            agent: agent.clone(),
            asset_id,
            prediction_type,
            value,
            confidence,
            submitted_at: env.ledger().timestamp(),
            resolution_ledger,
            status: PredictionStatus::Pending,
            resolved_value: None,
        };

        env.storage()
            .instance()
            .set(&DataKey::Prediction(prediction_id), &prediction);
        env.storage()
            .instance()
            .set(&DataKey::PredictionCount, &count);

        Ok(prediction_id)
    }

    /// Resolves a pending prediction with the actual observed value. Admin only.
    ///
    /// # Arguments
    /// * `prediction_id` — ID of the prediction to resolve.
    /// * `actual_value` — observed value scaled by 1_000_000.
    ///
    /// # Panics
    /// Panics on `NotInitialized`, `Unauthorized`, `PredictionNotFound`, or
    /// `PredictionNotPending`.
    pub fn resolve_prediction(
        env: Env,
        prediction_id: u64,
        actual_value: i128,
    ) -> Result<(), Error> {
        require_admin(&env)?;

        let key = DataKey::Prediction(prediction_id);
        let mut prediction: Prediction = env
            .storage()
            .instance()
            .get(&key)
            .ok_or(Error::PredictionNotFound)?;

        if prediction.status != PredictionStatus::Pending {
            return Err(Error::PredictionNotPending);
        }

        prediction.status = PredictionStatus::Resolved;
        prediction.resolved_value = Some(actual_value);
        env.storage().instance().set(&key, &prediction);
        Ok(())
    }

    /// Subscribes a user to an agent's predictions.
    ///
    /// # Arguments
    /// * `subscriber` — signing subscriber address.
    /// * `agent` — agent address to follow.
    ///
    /// # Panics
    /// Panics on `NotInitialized`, `Unauthorized`, or `AlreadySubscribed`.
    pub fn subscribe(env: Env, subscriber: Address, agent: Address) -> Result<(), Error> {
        require_initialized(&env)?;

        subscriber.require_auth();

        let agent_key = DataKey::AgentSubscribers(agent.clone());
        let mut agent_subs: Vec<Address> = env
            .storage()
            .instance()
            .get(&agent_key)
            .unwrap_or_else(|| Vec::new(&env));

        if vec_contains_address(&agent_subs, &subscriber) {
            return Err(Error::AlreadySubscribed);
        }
        agent_subs.push_back(subscriber.clone());

        let sub_key = DataKey::SubscriberAgents(subscriber.clone());
        let mut sub_agents: Vec<Address> = env
            .storage()
            .instance()
            .get(&sub_key)
            .unwrap_or_else(|| Vec::new(&env));
        sub_agents.push_back(agent.clone());

        env.storage().instance().set(&agent_key, &agent_subs);
        env.storage().instance().set(&sub_key, &sub_agents);
        Ok(())
    }

    /// Unsubscribes a user from an agent's predictions.
    ///
    /// # Arguments
    /// * `subscriber` — signing subscriber address.
    /// * `agent` — agent address to unfollow.
    ///
    /// # Panics
    /// Panics on `NotInitialized`, `Unauthorized`, or `NotSubscribed`.
    pub fn unsubscribe(env: Env, subscriber: Address, agent: Address) -> Result<(), Error> {
        require_initialized(&env)?;

        subscriber.require_auth();

        let agent_key = DataKey::AgentSubscribers(agent.clone());
        let agent_subs: Vec<Address> = env
            .storage()
            .instance()
            .get(&agent_key)
            .unwrap_or_else(|| Vec::new(&env));

        if !vec_contains_address(&agent_subs, &subscriber) {
            return Err(Error::NotSubscribed);
        }

        let updated_agent_subs = vec_remove_address(&env, &agent_subs, &subscriber);

        let sub_key = DataKey::SubscriberAgents(subscriber.clone());
        let sub_agents: Vec<Address> = env
            .storage()
            .instance()
            .get(&sub_key)
            .unwrap_or_else(|| Vec::new(&env));
        let updated_sub_agents = vec_remove_address(&env, &sub_agents, &agent);

        env.storage()
            .instance()
            .set(&agent_key, &updated_agent_subs);
        env.storage()
            .instance()
            .set(&sub_key, &updated_sub_agents);
        Ok(())
    }

    /// Returns a prediction by ID.
    ///
    /// # Arguments
    /// * `prediction_id` — prediction identifier.
    ///
    /// # Panics
    /// Panics on `NotInitialized` or `PredictionNotFound`.
    pub fn get_prediction(env: Env, prediction_id: u64) -> Result<Prediction, Error> {
        require_initialized(&env)?;

        env.storage()
            .instance()
            .get(&DataKey::Prediction(prediction_id))
            .ok_or(Error::PredictionNotFound)
    }

    /// Returns all predictions submitted by an agent.
    ///
    /// # Arguments
    /// * `agent` — agent address to query.
    ///
    /// # Panics
    /// Panics on `NotInitialized`.
    pub fn get_agent_predictions(env: Env, agent: Address) -> Result<Vec<Prediction>, Error> {
        require_initialized(&env)?;

        let count: u64 = env
            .storage()
            .instance()
            .get(&DataKey::PredictionCount)
            .unwrap_or(0);

        let mut predictions = Vec::new(&env);
        for id in 1..=count {
            if let Some(prediction) = env
                .storage()
                .instance()
                .get::<DataKey, Prediction>(&DataKey::Prediction(id))
            {
                if prediction.agent == agent {
                    predictions.push_back(prediction);
                }
            }
        }
        Ok(predictions)
    }

    /// Returns all agents a subscriber is following.
    ///
    /// # Arguments
    /// * `subscriber` — subscriber address to query.
    ///
    /// # Panics
    /// Panics on `NotInitialized`.
    pub fn get_subscriber_agents(env: Env, subscriber: Address) -> Result<Vec<Address>, Error> {
        require_initialized(&env)?;

        Ok(env
            .storage()
            .instance()
            .get(&DataKey::SubscriberAgents(subscriber))
            .unwrap_or_else(|| Vec::new(&env)))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use soroban_sdk::{
        symbol_short,
        testutils::{Address as _, Ledger},
        Env,
    };

    fn setup(env: &Env) -> (Address, Address, Address, PredictionMarketClient<'static>) {
        let admin = Address::generate(env);
        let reputation = Address::generate(env);
        let contract_id = env.register(PredictionMarket, ());
        let client = PredictionMarketClient::new(env, &contract_id);
        client.initialize(&admin, &reputation);
        (admin, reputation, contract_id, client)
    }

    fn submit_sample(env: &Env, client: &PredictionMarketClient, agent: &Address) -> u64 {
        env.ledger().set_sequence_number(100);
        client.submit_prediction(
            agent,
            &symbol_short!("REIT01"),
            &PredictionType::PriceTarget,
            &1_050_000_000,
            &8_500,
            &200,
        )
    }

    // --- initialize ---

    #[test]
    fn initialize_happy_path() {
        let env = Env::default();
        env.mock_all_auths();
        let admin = Address::generate(&env);
        let reputation = Address::generate(&env);
        let contract_id = env.register(PredictionMarket, ());
        let client = PredictionMarketClient::new(&env, &contract_id);

        client.initialize(&admin, &reputation);
        assert_eq!(client.get_subscriber_agents(&Address::generate(&env)).len(), 0);
    }

    #[test]
    fn initialize_fails_when_already_initialized() {
        let env = Env::default();
        env.mock_all_auths();
        let (admin, reputation, _id, client) = setup(&env);

        let result = client.try_initialize(&admin, &reputation);
        assert_eq!(result, Err(Ok(Error::AlreadyInitialized)));
    }

    #[test]
    fn initialize_sets_prediction_count_to_zero() {
        let env = Env::default();
        env.mock_all_auths();
        let (_admin, _rep, _id, client) = setup(&env);
        let agent = Address::generate(&env);
        assert_eq!(client.get_agent_predictions(&agent).len(), 0);
    }

    // --- submit_prediction ---

    #[test]
    fn submit_prediction_happy_path() {
        let env = Env::default();
        env.mock_all_auths();
        let (_admin, _rep, _id, client) = setup(&env);
        let agent = Address::generate(&env);

        let id = submit_sample(&env, &client, &agent);
        assert_eq!(id, 1);
        let pred = client.get_prediction(&id);
        assert_eq!(pred.agent, agent);
        assert_eq!(pred.status, PredictionStatus::Pending);
    }

    #[test]
    fn submit_prediction_fails_invalid_confidence() {
        let env = Env::default();
        env.mock_all_auths();
        let (_admin, _rep, _id, client) = setup(&env);
        let agent = Address::generate(&env);
        env.ledger().set_sequence_number(100);

        let result = client.try_submit_prediction(
            &agent,
            &symbol_short!("REIT01"),
            &PredictionType::PriceTarget,
            &1_000_000,
            &10_001,
            &200,
        );
        assert_eq!(result, Err(Ok(Error::InvalidConfidence)));
    }

    #[test]
    fn submit_prediction_fails_invalid_resolution_ledger() {
        let env = Env::default();
        env.mock_all_auths();
        let (_admin, _rep, _id, client) = setup(&env);
        let agent = Address::generate(&env);
        env.ledger().set_sequence_number(500);

        let result = client.try_submit_prediction(
            &agent,
            &symbol_short!("REIT01"),
            &PredictionType::PriceTarget,
            &1_000_000,
            &5_000,
            &100,
        );
        assert_eq!(result, Err(Ok(Error::InvalidResolutionLedger)));
    }

    #[test]
    fn submit_prediction_fails_when_unauthorized() {
        let env = Env::default();
        let (_admin, _rep, _id, client) = setup(&env);
        let agent = Address::generate(&env);
        env.ledger().set_sequence_number(100);

        let result = client.try_submit_prediction(
            &agent,
            &symbol_short!("REIT01"),
            &PredictionType::PriceTarget,
            &1_000_000,
            &5_000,
            &200,
        );
        assert!(matches!(result, Err(Err(_))));
    }

    // --- resolve_prediction ---

    #[test]
    fn resolve_prediction_happy_path() {
        let env = Env::default();
        env.mock_all_auths();
        let (_admin, _rep, _id, client) = setup(&env);
        let agent = Address::generate(&env);
        let id = submit_sample(&env, &client, &agent);

        client.resolve_prediction(&id, &1_040_000_000);
        let pred = client.get_prediction(&id);
        assert_eq!(pred.status, PredictionStatus::Resolved);
        assert_eq!(pred.resolved_value, Some(1_040_000_000));
    }

    #[test]
    fn resolve_prediction_fails_when_not_found() {
        let env = Env::default();
        env.mock_all_auths();
        let (_admin, _rep, _id, client) = setup(&env);

        let result = client.try_resolve_prediction(&999, &1_000_000);
        assert_eq!(result, Err(Ok(Error::PredictionNotFound)));
    }

    #[test]
    fn resolve_prediction_fails_when_not_pending() {
        let env = Env::default();
        env.mock_all_auths();
        let (_admin, _rep, _id, client) = setup(&env);
        let agent = Address::generate(&env);
        let id = submit_sample(&env, &client, &agent);
        client.resolve_prediction(&id, &1_000_000);

        let result = client.try_resolve_prediction(&id, &1_000_000);
        assert_eq!(result, Err(Ok(Error::PredictionNotPending)));
    }

    // --- subscribe ---

    #[test]
    fn subscribe_happy_path() {
        let env = Env::default();
        env.mock_all_auths();
        let (_admin, _rep, _id, client) = setup(&env);
        let subscriber = Address::generate(&env);
        let agent = Address::generate(&env);

        client.subscribe(&subscriber, &agent);
        let agents = client.get_subscriber_agents(&subscriber);
        assert_eq!(agents.len(), 1);
        assert_eq!(agents.get(0).unwrap(), agent);
    }

    #[test]
    fn subscribe_fails_when_already_subscribed() {
        let env = Env::default();
        env.mock_all_auths();
        let (_admin, _rep, _id, client) = setup(&env);
        let subscriber = Address::generate(&env);
        let agent = Address::generate(&env);

        client.subscribe(&subscriber, &agent);
        let result = client.try_subscribe(&subscriber, &agent);
        assert_eq!(result, Err(Ok(Error::AlreadySubscribed)));
    }

    #[test]
    fn subscribe_fails_when_unauthorized() {
        let env = Env::default();
        let (_admin, _rep, _id, client) = setup(&env);
        let subscriber = Address::generate(&env);
        let agent = Address::generate(&env);

        let result = client.try_subscribe(&subscriber, &agent);
        assert!(matches!(result, Err(Err(_))));
    }

    // --- unsubscribe ---

    #[test]
    fn unsubscribe_happy_path() {
        let env = Env::default();
        env.mock_all_auths();
        let (_admin, _rep, _id, client) = setup(&env);
        let subscriber = Address::generate(&env);
        let agent = Address::generate(&env);

        client.subscribe(&subscriber, &agent);
        client.unsubscribe(&subscriber, &agent);
        assert_eq!(client.get_subscriber_agents(&subscriber).len(), 0);
    }

    #[test]
    fn unsubscribe_fails_when_not_subscribed() {
        let env = Env::default();
        env.mock_all_auths();
        let (_admin, _rep, _id, client) = setup(&env);
        let subscriber = Address::generate(&env);
        let agent = Address::generate(&env);

        let result = client.try_unsubscribe(&subscriber, &agent);
        assert_eq!(result, Err(Ok(Error::NotSubscribed)));
    }

    #[test]
    fn unsubscribe_fails_when_unauthorized() {
        let env = Env::default();
        let (_admin, _rep, _id, client) = setup(&env);
        let subscriber = Address::generate(&env);
        let agent = Address::generate(&env);

        let result = client.try_unsubscribe(&subscriber, &agent);
        assert!(matches!(result, Err(Err(_))));
    }

    // --- get_prediction ---

    #[test]
    fn get_prediction_happy_path() {
        let env = Env::default();
        env.mock_all_auths();
        let (_admin, _rep, _id, client) = setup(&env);
        let agent = Address::generate(&env);
        let id = submit_sample(&env, &client, &agent);

        let pred = client.get_prediction(&id);
        assert_eq!(pred.id, id);
    }

    #[test]
    fn get_prediction_fails_when_not_found() {
        let env = Env::default();
        env.mock_all_auths();
        let (_admin, _rep, _id, client) = setup(&env);

        let result = client.try_get_prediction(&42);
        assert_eq!(result, Err(Ok(Error::PredictionNotFound)));
    }

    #[test]
    fn get_prediction_fails_when_not_initialized() {
        let env = Env::default();
        let contract_id = env.register(PredictionMarket, ());
        let client = PredictionMarketClient::new(&env, &contract_id);

        let result = client.try_get_prediction(&1);
        assert_eq!(result, Err(Ok(Error::NotInitialized)));
    }

    // --- get_agent_predictions ---

    #[test]
    fn get_agent_predictions_happy_path() {
        let env = Env::default();
        env.mock_all_auths();
        let (_admin, _rep, _id, client) = setup(&env);
        let agent = Address::generate(&env);
        submit_sample(&env, &client, &agent);
        submit_sample(&env, &client, &agent);

        assert_eq!(client.get_agent_predictions(&agent).len(), 2);
    }

    #[test]
    fn get_agent_predictions_empty_for_unknown_agent() {
        let env = Env::default();
        env.mock_all_auths();
        let (_admin, _rep, _id, client) = setup(&env);
        let agent = Address::generate(&env);

        assert_eq!(client.get_agent_predictions(&agent).len(), 0);
    }

    #[test]
    fn get_agent_predictions_fails_when_not_initialized() {
        let env = Env::default();
        let contract_id = env.register(PredictionMarket, ());
        let client = PredictionMarketClient::new(&env, &contract_id);
        let agent = Address::generate(&env);

        let result = client.try_get_agent_predictions(&agent);
        assert_eq!(result, Err(Ok(Error::NotInitialized)));
    }

    // --- get_subscriber_agents ---

    #[test]
    fn get_subscriber_agents_happy_path() {
        let env = Env::default();
        env.mock_all_auths();
        let (_admin, _rep, _id, client) = setup(&env);
        let subscriber = Address::generate(&env);
        let agent_a = Address::generate(&env);
        let agent_b = Address::generate(&env);

        client.subscribe(&subscriber, &agent_a);
        client.subscribe(&subscriber, &agent_b);
        assert_eq!(client.get_subscriber_agents(&subscriber).len(), 2);
    }

    #[test]
    fn get_subscriber_agents_returns_empty_by_default() {
        let env = Env::default();
        env.mock_all_auths();
        let (_admin, _rep, _id, client) = setup(&env);
        let subscriber = Address::generate(&env);

        assert_eq!(client.get_subscriber_agents(&subscriber).len(), 0);
    }

    #[test]
    fn get_subscriber_agents_fails_when_not_initialized() {
        let env = Env::default();
        let contract_id = env.register(PredictionMarket, ());
        let client = PredictionMarketClient::new(&env, &contract_id);
        let subscriber = Address::generate(&env);

        let result = client.try_get_subscriber_agents(&subscriber);
        assert_eq!(result, Err(Ok(Error::NotInitialized)));
    }
}
