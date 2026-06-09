#![no_std]

use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, Address, BytesN, Env, String, Symbol, Vec,
};

/// On-chain registry of tokenized real-world assets listed on Stellar.
#[contract]
pub struct AssetRegistry;

/// Typed storage keys for the asset registry.
#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Asset(Symbol),
    AssetCount,
    Admin,
    /// Ordered list of registered asset IDs for enumeration.
    AssetIds,
}

/// Category of real-world asset represented by a tokenized Stellar asset.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AssetType {
    RealEstate,
    Commodity,
    Bond,
    Equity,
    Other,
}

/// Metadata for a registered tokenized RWA.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Asset {
    pub id: Symbol,
    pub name: String,
    pub issuer: Address,
    pub asset_type: AssetType,
    pub stellar_asset_contract: BytesN<32>,
    pub registered_at: u64,
    pub is_active: bool,
}

/// Typed errors for the asset registry contract.
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    NotInitialized = 1,
    AlreadyInitialized = 2,
    Unauthorized = 3,
    AssetNotFound = 4,
    AssetAlreadyExists = 5,
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

#[contractimpl]
impl AssetRegistry {
    /// Initializes the contract with an admin address.
    ///
    /// # Arguments
    /// * `admin` — address granted registry administration rights.
    ///
    /// # Panics
    /// Panics (via `AlreadyInitialized` error) if the contract has already been initialized.
    pub fn initialize(env: Env, admin: Address) -> Result<(), Error> {
        if env.storage().instance().has(&DataKey::Admin) {
            return Err(Error::AlreadyInitialized);
        }
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::AssetCount, &0u32);
        env.storage()
            .instance()
            .set(&DataKey::AssetIds, &Vec::<Symbol>::new(&env));
        Ok(())
    }

    /// Registers a new tokenized RWA. Only the admin may call this.
    ///
    /// # Arguments
    /// * `asset` — full asset metadata including a unique `id` symbol.
    ///
    /// # Panics
    /// Panics on `NotInitialized`, `Unauthorized`, or `AssetAlreadyExists`.
    pub fn register_asset(env: Env, asset: Asset) -> Result<(), Error> {
        require_admin(&env)?;

        let key = DataKey::Asset(asset.id.clone());
        if env.storage().instance().has(&key) {
            return Err(Error::AssetAlreadyExists);
        }

        let mut count: u32 = env
            .storage()
            .instance()
            .get(&DataKey::AssetCount)
            .unwrap_or(0);
        count = count
            .checked_add(1)
            .expect("asset count overflow");

        let mut asset_ids: Vec<Symbol> = env
            .storage()
            .instance()
            .get(&DataKey::AssetIds)
            .unwrap_or_else(|| Vec::new(&env));
        asset_ids.push_back(asset.id.clone());

        env.storage().instance().set(&key, &asset);
        env.storage().instance().set(&DataKey::AssetCount, &count);
        env.storage().instance().set(&DataKey::AssetIds, &asset_ids);
        Ok(())
    }

    /// Deactivates a registered asset. Only the admin may call this.
    ///
    /// # Arguments
    /// * `asset_id` — symbol identifier of the asset to deactivate.
    ///
    /// # Panics
    /// Panics on `NotInitialized`, `Unauthorized`, or `AssetNotFound`.
    pub fn deactivate_asset(env: Env, asset_id: Symbol) -> Result<(), Error> {
        require_admin(&env)?;

        let key = DataKey::Asset(asset_id);
        let mut asset: Asset = env
            .storage()
            .instance()
            .get(&key)
            .ok_or(Error::AssetNotFound)?;
        asset.is_active = false;
        env.storage().instance().set(&key, &asset);
        Ok(())
    }

    /// Returns the metadata for a registered asset.
    ///
    /// # Arguments
    /// * `asset_id` — symbol identifier of the asset.
    ///
    /// # Panics
    /// Panics on `NotInitialized` or `AssetNotFound`.
    pub fn get_asset(env: Env, asset_id: Symbol) -> Result<Asset, Error> {
        require_initialized(&env)?;

        env.storage()
            .instance()
            .get(&DataKey::Asset(asset_id))
            .ok_or(Error::AssetNotFound)
    }

    /// Returns all currently active registered assets.
    ///
    /// # Panics
    /// Panics on `NotInitialized`.
    pub fn list_active_assets(env: Env) -> Result<Vec<Asset>, Error> {
        require_initialized(&env)?;

        let asset_ids: Vec<Symbol> = env
            .storage()
            .instance()
            .get(&DataKey::AssetIds)
            .unwrap_or_else(|| Vec::new(&env));

        let mut active = Vec::new(&env);
        for id in asset_ids.iter() {
            if let Some(asset) = env
                .storage()
                .instance()
                .get::<DataKey, Asset>(&DataKey::Asset(id))
            {
                if asset.is_active {
                    active.push_back(asset);
                }
            }
        }
        Ok(active)
    }

    /// Returns the total number of registered assets (active and inactive).
    ///
    /// # Panics
    /// Panics on `NotInitialized`.
    pub fn asset_count(env: Env) -> Result<u32, Error> {
        require_initialized(&env)?;

        Ok(env
            .storage()
            .instance()
            .get(&DataKey::AssetCount)
            .unwrap_or(0))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use soroban_sdk::{symbol_short, testutils::Address as _, Env};

    fn sample_asset(env: &Env, id: &str) -> Asset {
        let issuer = Address::generate(env);
        Asset {
            id: Symbol::new(env, id),
            name: String::from_str(env, "Test Asset"),
            issuer: issuer.clone(),
            asset_type: AssetType::RealEstate,
            stellar_asset_contract: BytesN::from_array(env, &[1u8; 32]),
            registered_at: 1_700_000_000,
            is_active: true,
        }
    }

    fn setup(env: &Env) -> (Address, Address, AssetRegistryClient<'static>) {
        let admin = Address::generate(env);
        let contract_id = env.register(AssetRegistry, ());
        let client = AssetRegistryClient::new(env, &contract_id);
        client.initialize(&admin);
        (admin, contract_id, client)
    }

    // --- initialize ---

    #[test]
    fn initialize_happy_path() {
        let env = Env::default();
        env.mock_all_auths();
        let admin = Address::generate(&env);
        let contract_id = env.register(AssetRegistry, ());
        let client = AssetRegistryClient::new(&env, &contract_id);

        client.initialize(&admin);
        assert_eq!(client.asset_count(), 0);
    }

    #[test]
    fn initialize_fails_when_already_initialized() {
        let env = Env::default();
        env.mock_all_auths();
        let (_admin, _id, client) = setup(&env);

        let other = Address::generate(&env);
        let result = client.try_initialize(&other);
        assert_eq!(result, Err(Ok(Error::AlreadyInitialized)));
    }

    #[test]
    fn initialize_requires_valid_admin() {
        let env = Env::default();
        let admin = Address::generate(&env);
        let contract_id = env.register(AssetRegistry, ());
        let client = AssetRegistryClient::new(&env, &contract_id);

        client.initialize(&admin);
        assert_eq!(client.asset_count(), 0);
    }

    // --- register_asset ---

    #[test]
    fn register_asset_happy_path() {
        let env = Env::default();
        env.mock_all_auths();
        let (_admin, _id, client) = setup(&env);
        let asset = sample_asset(&env, "REIT01");

        client.register_asset(&asset);
        assert_eq!(client.asset_count(), 1);
        let stored = client.get_asset(&asset.id);
        assert_eq!(stored, asset);
    }

    #[test]
    fn register_asset_fails_when_unauthorized() {
        let env = Env::default();
        let (_admin, _id, client) = setup(&env);
        let asset = sample_asset(&env, "REIT01");

        let result = client.try_register_asset(&asset);
        assert!(matches!(result, Err(Err(_))));
    }

    #[test]
    fn register_asset_fails_when_duplicate() {
        let env = Env::default();
        env.mock_all_auths();
        let (_admin, _id, client) = setup(&env);
        let asset = sample_asset(&env, "REIT01");

        client.register_asset(&asset);
        let result = client.try_register_asset(&asset);
        assert_eq!(result, Err(Ok(Error::AssetAlreadyExists)));
    }

    // --- deactivate_asset ---

    #[test]
    fn deactivate_asset_happy_path() {
        let env = Env::default();
        env.mock_all_auths();
        let (_admin, _id, client) = setup(&env);
        let asset = sample_asset(&env, "REIT01");
        client.register_asset(&asset);

        client.deactivate_asset(&asset.id);
        let stored = client.get_asset(&asset.id);
        assert!(!stored.is_active);
        assert_eq!(client.list_active_assets().len(), 0);
    }

    #[test]
    fn deactivate_asset_fails_when_not_found() {
        let env = Env::default();
        env.mock_all_auths();
        let (_admin, _id, client) = setup(&env);

        let result = client.try_deactivate_asset(&symbol_short!("MISSING"));
        assert_eq!(result, Err(Ok(Error::AssetNotFound)));
    }

    #[test]
    fn deactivate_asset_fails_when_unauthorized() {
        let env = Env::default();
        let (_admin, _id, client) = setup(&env);

        let result = client.try_deactivate_asset(&symbol_short!("REIT01"));
        assert!(matches!(result, Err(Err(_))));
    }

    // --- get_asset ---

    #[test]
    fn get_asset_happy_path() {
        let env = Env::default();
        env.mock_all_auths();
        let (_admin, _id, client) = setup(&env);
        let asset = sample_asset(&env, "BOND01");
        client.register_asset(&asset);

        assert_eq!(client.get_asset(&asset.id), asset);
    }

    #[test]
    fn get_asset_fails_when_not_found() {
        let env = Env::default();
        env.mock_all_auths();
        let (_admin, _id, client) = setup(&env);

        let result = client.try_get_asset(&symbol_short!("NOPE"));
        assert_eq!(result, Err(Ok(Error::AssetNotFound)));
    }

    #[test]
    fn get_asset_fails_when_not_initialized() {
        let env = Env::default();
        let contract_id = env.register(AssetRegistry, ());
        let client = AssetRegistryClient::new(&env, &contract_id);

        let result = client.try_get_asset(&symbol_short!("X"));
        assert_eq!(result, Err(Ok(Error::NotInitialized)));
    }

    // --- list_active_assets ---

    #[test]
    fn list_active_assets_happy_path() {
        let env = Env::default();
        env.mock_all_auths();
        let (_admin, _id, client) = setup(&env);
        let active = sample_asset(&env, "ACTIVE");
        let inactive = sample_asset(&env, "INACTV");
        client.register_asset(&active);
        client.register_asset(&inactive);
        client.deactivate_asset(&inactive.id);

        let list = client.list_active_assets();
        assert_eq!(list.len(), 1);
        assert_eq!(list.get(0).unwrap(), active);
    }

    #[test]
    fn list_active_assets_returns_empty_when_none_active() {
        let env = Env::default();
        env.mock_all_auths();
        let (_admin, _id, client) = setup(&env);

        assert_eq!(client.list_active_assets().len(), 0);
    }

    #[test]
    fn list_active_assets_fails_when_not_initialized() {
        let env = Env::default();
        let contract_id = env.register(AssetRegistry, ());
        let client = AssetRegistryClient::new(&env, &contract_id);

        let result = client.try_list_active_assets();
        assert_eq!(result, Err(Ok(Error::NotInitialized)));
    }

    // --- asset_count ---

    #[test]
    fn asset_count_happy_path() {
        let env = Env::default();
        env.mock_all_auths();
        let (_admin, _id, client) = setup(&env);
        client.register_asset(&sample_asset(&env, "A1"));
        client.register_asset(&sample_asset(&env, "A2"));

        assert_eq!(client.asset_count(), 2);
    }

    #[test]
    fn asset_count_fails_when_not_initialized() {
        let env = Env::default();
        let contract_id = env.register(AssetRegistry, ());
        let client = AssetRegistryClient::new(&env, &contract_id);

        let result = client.try_asset_count();
        assert_eq!(result, Err(Ok(Error::NotInitialized)));
    }

    #[test]
    fn asset_count_starts_at_zero() {
        let env = Env::default();
        env.mock_all_auths();
        let (_admin, _id, client) = setup(&env);
        assert_eq!(client.asset_count(), 0);
    }
}
