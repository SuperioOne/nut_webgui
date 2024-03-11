use crate::http_server::hypermedia::ups_info::UpsInfo;
use crate::http_server::ServerState;
use askama::Template;
use axum::extract::{Query, State};
use axum_core::response::IntoResponse;
use serde::Deserialize;
use std::sync::Arc;

#[derive(Template)]
#[template(path = "ups_table.html", ext = "html")]
struct UpsTableTemplate {
  ups_list: Vec<UpsInfo>,
}

#[derive(Deserialize)]
pub struct HomeQuery {
  section: Option<String>,
}

#[derive(Template)]
#[template(path = "+page.html", escape = "none", ext = "html")]
struct HomeTemplate {
  title: String,
  ups_table: UpsTableTemplate,
}

pub async fn get(
  query: Query<HomeQuery>,
  State(state): State<Arc<ServerState>>,
) -> impl IntoResponse {
  let ups_list: Vec<UpsInfo> = state
    .store
    .read()
    .await
    .into_iter()
    .map(|(_, ups)| UpsInfo::from(ups))
    .collect();

  let table_template = UpsTableTemplate { ups_list };

  if let Some("ups_table") = query.section.as_deref() {
    table_template.into_response()
  } else {
    HomeTemplate {
      title: "Home".to_string(),
      ups_table: table_template,
    }
    .into_response()
  }
}
