use crate::{
  auth::user_session::UserSession,
  http::hypermedia::{
    error::ErrorPage,
    util::{
      named_enum::create_named_enum,
      render_extra::RenderWithConfig,
      topology_graph::{GraphNode as _, TopologyGraph, TopologyGraphBuilder},
      unit::UnitDisplay as _,
    },
  },
  state::ServerState,
};
use askama::Template;
use axum::{
  Extension,
  extract::{Query, State},
  response::{Html, IntoResponse, Response},
};
use serde::Deserialize;
use std::{ops::Deref, sync::Arc};
use upsmc::ups_status::UpsStatus;

create_named_enum!(
    #[derive(Debug, Copy, Clone, PartialEq, Eq)]
    named_enum TopologyPageSection {
        GraphNodes = "graph_nodes"
    }
);

#[derive(Template)]
#[template(path = "topology/+page.html", blocks = ["graph_nodes"])]
struct TopologyTemplate {
  graph: TopologyGraph,
}

#[derive(Deserialize, Clone, Copy)]
pub struct TopologyFragmentQuery {
  section: Option<TopologyPageSection>,
}

pub async fn get(
  query: Query<TopologyFragmentQuery>,
  state: State<Arc<ServerState>>,
  session: Option<Extension<UserSession>>,
) -> Result<Response, ErrorPage> {
  let mut graph_builder = TopologyGraphBuilder::new();

  for (namespace, upsd) in state.upsd_servers.iter() {
    let state = upsd.daemon_state.read().await;
    graph_builder.add_upsd_server(
      namespace.clone(),
      upsd.config.addr.clone(),
      upsd.config.port,
      state.deref(),
    );
  }

  let session = session.map(|v| v.0);
  let template = TopologyTemplate {
    graph: graph_builder.finish(),
  };
  let body = match query.section {
    Some(TopologyPageSection::GraphNodes) => template
      .as_graph_nodes()
      .render_with_config(&state.config, session.as_ref()),
    _ => template.render_with_config(&state.config, session.as_ref()),
  }?;

  Ok(Html(body).into_response())
}
