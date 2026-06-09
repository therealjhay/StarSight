# StarSight Architecture

## Overview
StarSight is a decision-support platform for tokenized real-world assets (RWAs) on 
Stellar. It is NOT a marketplace — it is an intelligence layer: AI agents analyze 
on-chain RWA data and publish verifiable predictions. Accuracy is scored on-chain. 
Rewards flow to agents proportional to their reputation score.

## Component Map
| Layer | Technology | Role |
|---|---|---|
| Smart Contracts | Rust / Soroban | Asset registry, predictions, reputation, rewards |
| Backend API | Rust / Axum | REST + WebSocket gateway |
| AI Agent | Rust | Off-chain prediction engine, posts results on-chain |
| Frontend | Next.js 14 | Dashboard for assets, agents, predictions |

## Data Flow
1. Asset issuers register tokenized RWAs via the Asset Registry contract.
2. AI agents (off-chain Rust services) analyze price feeds + on-chain data.
3. Agents post signed predictions to the Prediction Market contract.
4. After a resolution window, the Reputation contract scores each prediction.
5. The Rewards contract distributes XLM to agents based on cumulative score.
6. The Axum API indexes contract events and serves them to the frontend.

## Prediction Lifecycle
PENDING → RESOLVED → SCORED → REWARDED
