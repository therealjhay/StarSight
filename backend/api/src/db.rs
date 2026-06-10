#![allow(dead_code)]

use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;

use crate::models::{AgentScore, Asset, Prediction};

/// Creates a PostgreSQL connection pool.
///
/// # Panics
/// Panics if the connection cannot be established.
pub async fn create_pool(database_url: &str) -> PgPool {
    PgPoolOptions::new()
        .max_connections(10)
        .connect(database_url)
        .await
        .expect("Failed to create PostgreSQL pool")
}

/// Runs inline SQL migrations to create tables if they do not exist.
pub async fn run_migrations(pool: &PgPool) {
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS assets (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            issuer TEXT NOT NULL,
            asset_type TEXT NOT NULL,
            stellar_asset_contract TEXT NOT NULL,
            registered_at BIGINT NOT NULL,
            is_active BOOLEAN NOT NULL DEFAULT TRUE
        );
        "#,
    )
    .execute(pool)
    .await
    .expect("Failed to create assets table");

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS predictions (
            id BIGINT PRIMARY KEY,
            agent TEXT NOT NULL,
            asset_id TEXT NOT NULL,
            prediction_type TEXT NOT NULL,
            value BIGINT NOT NULL,
            confidence INT NOT NULL,
            submitted_at BIGINT NOT NULL,
            resolution_ledger BIGINT NOT NULL,
            status TEXT NOT NULL DEFAULT 'Pending',
            resolved_value BIGINT
        );
        "#,
    )
    .execute(pool)
    .await
    .expect("Failed to create predictions table");

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS agent_scores (
            agent TEXT PRIMARY KEY,
            total_predictions INT NOT NULL DEFAULT 0,
            correct_predictions INT NOT NULL DEFAULT 0,
            accuracy_bps INT NOT NULL DEFAULT 0,
            streak INT NOT NULL DEFAULT 0,
            last_scored_at BIGINT NOT NULL DEFAULT 0
        );
        "#,
    )
    .execute(pool)
    .await
    .expect("Failed to create agent_scores table");

    tracing::info!("Database migrations applied successfully");
}

// ---------------------------------------------------------------------------
// Asset queries
// ---------------------------------------------------------------------------

/// Returns all assets.
pub async fn get_all_assets(pool: &PgPool) -> Result<Vec<Asset>, sqlx::Error> {
    sqlx::query_as::<_, Asset>("SELECT * FROM assets ORDER BY registered_at DESC")
        .fetch_all(pool)
        .await
}

/// Returns a single asset by ID.
pub async fn get_asset_by_id(pool: &PgPool, id: &str) -> Result<Option<Asset>, sqlx::Error> {
    sqlx::query_as::<_, Asset>("SELECT * FROM assets WHERE id = $1")
        .bind(id)
        .fetch_optional(pool)
        .await
}

/// Upserts an asset row.
pub async fn upsert_asset(pool: &PgPool, asset: &Asset) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO assets (id, name, issuer, asset_type, stellar_asset_contract, registered_at, is_active)
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        ON CONFLICT (id) DO UPDATE SET
            name = EXCLUDED.name,
            issuer = EXCLUDED.issuer,
            asset_type = EXCLUDED.asset_type,
            stellar_asset_contract = EXCLUDED.stellar_asset_contract,
            registered_at = EXCLUDED.registered_at,
            is_active = EXCLUDED.is_active
        "#,
    )
    .bind(&asset.id)
    .bind(&asset.name)
    .bind(&asset.issuer)
    .bind(&asset.asset_type)
    .bind(&asset.stellar_asset_contract)
    .bind(asset.registered_at)
    .bind(asset.is_active)
    .execute(pool)
    .await?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Prediction queries
// ---------------------------------------------------------------------------

/// Returns all predictions, newest first.
pub async fn get_all_predictions(pool: &PgPool) -> Result<Vec<Prediction>, sqlx::Error> {
    sqlx::query_as::<_, Prediction>("SELECT * FROM predictions ORDER BY submitted_at DESC")
        .fetch_all(pool)
        .await
}

/// Returns a single prediction by ID.
pub async fn get_prediction_by_id(
    pool: &PgPool,
    id: i64,
) -> Result<Option<Prediction>, sqlx::Error> {
    sqlx::query_as::<_, Prediction>("SELECT * FROM predictions WHERE id = $1")
        .bind(id)
        .fetch_optional(pool)
        .await
}

/// Returns all predictions by a given agent address.
pub async fn get_predictions_by_agent(
    pool: &PgPool,
    agent: &str,
) -> Result<Vec<Prediction>, sqlx::Error> {
    sqlx::query_as::<_, Prediction>(
        "SELECT * FROM predictions WHERE agent = $1 ORDER BY submitted_at DESC",
    )
    .bind(agent)
    .fetch_all(pool)
    .await
}

/// Upserts a prediction row.
pub async fn upsert_prediction(pool: &PgPool, pred: &Prediction) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO predictions (id, agent, asset_id, prediction_type, value, confidence, submitted_at, resolution_ledger, status, resolved_value)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
        ON CONFLICT (id) DO UPDATE SET
            agent = EXCLUDED.agent,
            asset_id = EXCLUDED.asset_id,
            prediction_type = EXCLUDED.prediction_type,
            value = EXCLUDED.value,
            confidence = EXCLUDED.confidence,
            submitted_at = EXCLUDED.submitted_at,
            resolution_ledger = EXCLUDED.resolution_ledger,
            status = EXCLUDED.status,
            resolved_value = EXCLUDED.resolved_value
        "#,
    )
    .bind(pred.id)
    .bind(&pred.agent)
    .bind(&pred.asset_id)
    .bind(&pred.prediction_type)
    .bind(pred.value)
    .bind(pred.confidence)
    .bind(pred.submitted_at)
    .bind(pred.resolution_ledger)
    .bind(&pred.status)
    .bind(pred.resolved_value)
    .execute(pool)
    .await?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Agent score queries
// ---------------------------------------------------------------------------

/// Returns all agent scores.
pub async fn get_all_agent_scores(pool: &PgPool) -> Result<Vec<AgentScore>, sqlx::Error> {
    sqlx::query_as::<_, AgentScore>("SELECT * FROM agent_scores ORDER BY accuracy_bps DESC")
        .fetch_all(pool)
        .await
}

/// Returns a single agent's score by address.
pub async fn get_agent_score(
    pool: &PgPool,
    agent: &str,
) -> Result<Option<AgentScore>, sqlx::Error> {
    sqlx::query_as::<_, AgentScore>("SELECT * FROM agent_scores WHERE agent = $1")
        .bind(agent)
        .fetch_optional(pool)
        .await
}

/// Upserts an agent score row.
pub async fn upsert_agent_score(pool: &PgPool, score: &AgentScore) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO agent_scores (agent, total_predictions, correct_predictions, accuracy_bps, streak, last_scored_at)
        VALUES ($1, $2, $3, $4, $5, $6)
        ON CONFLICT (agent) DO UPDATE SET
            total_predictions = EXCLUDED.total_predictions,
            correct_predictions = EXCLUDED.correct_predictions,
            accuracy_bps = EXCLUDED.accuracy_bps,
            streak = EXCLUDED.streak,
            last_scored_at = EXCLUDED.last_scored_at
        "#,
    )
    .bind(&score.agent)
    .bind(score.total_predictions)
    .bind(score.correct_predictions)
    .bind(score.accuracy_bps)
    .bind(score.streak)
    .bind(score.last_scored_at)
    .execute(pool)
    .await?;
    Ok(())
}
