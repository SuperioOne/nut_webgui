use crate::http_server::ServerState;
use askama::Template;
use axum::extract::State;
use std::sync::Arc;

pub async fn post(State(state): State<Arc<ServerState>>) {}

pub async fn get(State(state): State<Arc<ServerState>>) {}
