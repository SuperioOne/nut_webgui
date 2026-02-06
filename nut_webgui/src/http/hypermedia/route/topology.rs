use crate::{
  auth::user_session::UserSession,
  http::hypermedia::{
    error::ErrorPage,
    semantic_type::SemanticType,
    unit::{Percentage, RemainingSeconds, UnitDisplay, Voltage},
    util::RenderWithConfig,
  },
  state::{ConnectionStatus, DeviceEntry, ServerState, UpsdNamespace},
};
use askama::Template;
use axum::{
  Extension,
  extract::{Query, State},
  response::{Html, IntoResponse, Response},
};
use nut_webgui_upsmc::{UpsName, VarName, ups_status::UpsStatus};
use serde::Deserialize;
use std::{collections::BTreeMap, net::IpAddr, sync::Arc};

#[derive(Template)]
#[template(path = "topology/+page.html", blocks = ["graph_nodes"])]
struct TopologyTemplate {
  graph: TopologyGraph,
}

#[derive(Deserialize)]
pub struct TopologyFragmentQuery {
  section: Option<Box<str>>,
}

pub async fn get(
  query: Query<TopologyFragmentQuery>,
  state: State<Arc<ServerState>>,
  session: Option<Extension<UserSession>>,
) -> Result<Response, ErrorPage> {
  let graph = TopologyGraphBuilder::from_server_state(state.0.clone())
    .await
    .finish();

  let session = session.map(|v| v.0);
  let template = TopologyTemplate { graph };

  let html = match query.section.as_deref() {
    Some("graph_nodes") => Html(
      template
        .as_graph_nodes()
        .render_with_config(&state.config, session.as_ref())?,
    ),
    _ => Html(template.render_with_config(&state.config, session.as_ref())?),
  };

  Ok(html.into_response())
}

struct TopologyGraph {
  pub servers: BTreeMap<UpsdNamespace, NutServerNode>,
  pub clients: BTreeMap<IpAddr, ClientNode>,
  pub devices: BTreeMap<Arc<DeviceId>, DeviceNode>,
  pub edges: Vec<EdgeNode>,
}

struct TopologyGraphBuilder {
  servers: BTreeMap<UpsdNamespace, NutServerNode>,
  clients: BTreeMap<IpAddr, ClientNode>,
  devices: BTreeMap<Arc<DeviceId>, DeviceNode>,
  edge_refs: Vec<EdgeRefNode>,
}

impl TopologyGraphBuilder {
  #[inline]
  pub fn new() -> Self {
    Self {
      servers: BTreeMap::new(),
      clients: BTreeMap::new(),
      devices: BTreeMap::new(),
      edge_refs: Vec::new(),
    }
  }

  pub async fn from_server_state(state: Arc<ServerState>) -> Self {
    let mut graph = Self::new();

    for (namespace, upsd) in state.upsd_servers.iter() {
      let daemon_state = upsd.daemon_state.read().await;

      let mut server_node = NutServerNode {
        namespace: namespace.clone(),
        port: upsd.config.port,
        status: daemon_state.status,
        address: upsd.config.addr.clone(),
        connection_counter: ConnectionCounter::default(),
        id: 0,
      };

      for device in daemon_state.devices.values() {
        _ = graph.insert_from(device, &mut server_node);
      }

      _ = graph
        .servers
        .insert(server_node.namespace.clone(), server_node);
    }

    graph
  }

  pub fn finish(mut self) -> TopologyGraph {
    for (pos, server) in self.servers.values_mut().enumerate() {
      server.id = pos;
    }

    for (pos, client) in self.clients.values_mut().enumerate() {
      client.id = pos;
    }

    for (pos, device) in self.devices.values_mut().enumerate() {
      device.id = pos;
    }

    let mut edges: Vec<EdgeNode> = Vec::with_capacity(self.edge_refs.len());

    for (id, edge) in self.edge_refs.iter().enumerate() {
      let from = self.resolve_link(&edge.from);
      let to = self.resolve_link(&edge.to);

      match (from, to) {
        (Some(from), Some(to)) => {
          edges.push(EdgeNode {
            id,
            semantic_type: edge.semantic_type,
            from,
            to,
            edge_type: edge.edge_type,
          });
        }
        _ => continue,
      }
    }

    edges.sort_by(|a, b| match a.from.cmp(&b.from) {
      std::cmp::Ordering::Equal => a.to.cmp(&b.to),
      cmp => cmp,
    });

    TopologyGraph {
      devices: self.devices,
      edges,
      clients: self.clients,
      servers: self.servers,
    }
  }

  fn resolve_link(&self, link: &Link) -> Option<NodeId> {
    match link {
      Link::Device(key) => self.devices.get(key).map(|v| v.get_id()),
      Link::Client(key) => self.clients.get(key).map(|v| v.get_id()),
      Link::Server(key) => self.servers.get(key).map(|v| v.get_id()),
    }
  }

  fn insert_from(&mut self, device: &DeviceEntry, server_node: &mut NutServerNode) {
    let key = DeviceId {
      name: device.name.to_owned(),
      namespace: server_node.namespace.clone(),
      model: device
        .variables
        .get(VarName::UPS_MODEL)
        .map(|v| v.to_string()),
      serial: device
        .variables
        .get(VarName::UPS_SERIAL)
        .map(|v| v.to_string()),
      manufacturer: device
        .variables
        .get(VarName::UPS_MFR)
        .map(|v| v.to_string()),
    };

    match self.devices.get_mut(&key) {
      Some(node) => {
        node.aliases.push(DeviceAlias {
          name: device.name.to_owned(),
          namespace: server_node.namespace.clone(),
        });

        self.edge_refs.push(EdgeRefNode {
          to: server_node.namespace.clone().into(),
          from: node.device_id.clone().into(),
          edge_type: EdgeType::Data,
          semantic_type: if device.status.has(UpsStatus::NOCOMM) {
            SemanticType::Error
          } else {
            SemanticType::Info
          },
        });
      }
      None => {
        let key = Arc::new(key);
        let semantic_type = {
          if device.status.has(UpsStatus::NOCOMM)
            || device.status.has(UpsStatus::OFFLINE)
            || device.status.has(UpsStatus::BYPASS)
          {
            SemanticType::None
          } else if device.status.has(UpsStatus::FORCED_SHUTDOWN)
            || device.status.has(UpsStatus::LOW_BATTERY)
            || device.status.has(UpsStatus::REPLACE_BATTERY)
            || device.status.has(UpsStatus::ALARM)
            || device.status.has(UpsStatus::OVERLOADED)
          {
            SemanticType::Error
          } else if device.status.has(UpsStatus::ON_BATTERY)
            || device.status.has(UpsStatus::TRIM)
            || device.status.has(UpsStatus::BOOST)
          {
            SemanticType::Warning
          } else {
            SemanticType::Success
          }
        };

        let device_node = DeviceNode {
          id: 0,
          device_id: key.clone(),
          aliases: vec![DeviceAlias {
            name: device.name.to_owned(),
            namespace: server_node.namespace.clone(),
          }],
          battery: device.get_battery_charge(),
          input_voltage: device.get_input_voltage(),
          load: device.get_ups_load(),
          runtime: device.get_battery_runtime(),
          semantic_type,
          status: device.status,
        };

        for client in device.attached.iter() {
          if client.addr.is_loopback() {
            match semantic_type {
              SemanticType::None => {}
              SemanticType::Error => server_node.connection_counter.increment_failed(),
              SemanticType::Warning => server_node.connection_counter.increment_degraded(),
              _ => server_node.connection_counter.increment_active(),
            };

            self.edge_refs.push(EdgeRefNode {
              to: server_node.namespace.clone().into(),
              from: key.clone().into(),
              edge_type: EdgeType::Power,
              semantic_type,
            });
          } else {
            let client = self
              .clients
              .entry(client.addr)
              .or_insert_with(|| ClientNode::new(client.addr, client.name.clone()));

            match semantic_type {
              SemanticType::None => {}
              SemanticType::Error => client.connection_counter.increment_failed(),
              SemanticType::Warning => client.connection_counter.increment_degraded(),
              _ => client.connection_counter.increment_active(),
            };

            self.edge_refs.push(EdgeRefNode {
              to: client.address.into(),
              from: key.clone().into(),
              edge_type: EdgeType::Power,
              semantic_type: semantic_type,
            });
          }
        }

        self.edge_refs.push(EdgeRefNode {
          to: server_node.namespace.clone().into(),
          from: key.clone().into(),
          edge_type: EdgeType::Data,
          semantic_type: if device.status.has(UpsStatus::NOCOMM) {
            SemanticType::Error
          } else {
            SemanticType::Info
          },
        });

        self.devices.insert(key, device_node);
      }
    }
  }
}

trait GraphNode {
  fn get_semantic_type(&self) -> SemanticType;
  fn get_id(&self) -> NodeId;
}

#[derive(Default)]
struct ConnectionCounter {
  active: usize,
  degraded: usize,
  failed: usize,
}

#[derive(Eq, PartialEq)]
struct DeviceId {
  pub manufacturer: Option<String>,
  pub model: Option<String>,
  pub serial: Option<String>,
  pub name: UpsName,
  pub namespace: UpsdNamespace,
}

struct DeviceAlias {
  pub name: UpsName,
  pub namespace: Arc<str>,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
enum NodeType {
  Server,
  Client,
  Device,
  Edge,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct NodeId {
  id: usize,
  node_type: NodeType,
}

#[derive(Copy, Clone, PartialEq, PartialOrd)]
enum EdgeType {
  Data,
  Power,
}

#[derive(Clone)]
enum Link {
  Device(Arc<DeviceId>),
  Client(IpAddr),
  Server(Arc<str>),
}

struct ClientNode {
  connection_counter: ConnectionCounter,
  id: usize,
  pub address: IpAddr,
  pub name: Option<Box<str>>,
}

struct NutServerNode {
  connection_counter: ConnectionCounter,
  id: usize,
  pub namespace: Arc<str>,
  pub address: Box<str>,
  pub port: u16,
  pub status: ConnectionStatus,
}

struct DeviceNode {
  id: usize,
  semantic_type: SemanticType,
  pub aliases: Vec<DeviceAlias>,
  pub battery: Option<Percentage>,
  pub device_id: Arc<DeviceId>,
  pub input_voltage: Option<Voltage>,
  pub load: Option<Percentage>,
  pub runtime: Option<RemainingSeconds>,
  pub status: UpsStatus,
}

struct EdgeRefNode {
  semantic_type: SemanticType,
  pub from: Link,
  pub to: Link,
  pub edge_type: EdgeType,
}

struct EdgeNode {
  semantic_type: SemanticType,
  id: usize,
  pub from: NodeId,
  pub to: NodeId,
  pub edge_type: EdgeType,
}

impl EdgeType {
  pub const fn as_class(&self) -> &'static str {
    match self {
      EdgeType::Power => "edge-dashed",
      EdgeType::Data => "edge-dotted",
    }
  }
}

impl NodeType {
  pub const fn as_str(&self) -> &'static str {
    match self {
      NodeType::Server => "server",
      NodeType::Client => "client",
      NodeType::Device => "device",
      NodeType::Edge => "edge",
    }
  }
}

impl ClientNode {
  pub fn new(address: IpAddr, name: Option<Box<str>>) -> Self {
    Self {
      id: 0,
      address,
      name,
      connection_counter: ConnectionCounter::default(),
    }
  }
}

impl PartialOrd for DeviceId {
  #[inline]
  fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
    Some(self.cmp(other))
  }
}

impl Ord for DeviceId {
  fn cmp(&self, other: &Self) -> std::cmp::Ordering {
    match self.manufacturer.cmp(&other.manufacturer) {
      core::cmp::Ordering::Equal => {}
      ord => return ord,
    }

    match self.model.cmp(&other.model) {
      core::cmp::Ordering::Equal => {}
      ord => return ord,
    }

    match self.serial.cmp(&other.serial) {
      core::cmp::Ordering::Equal => {
        if self.serial.is_none() && self.model.is_none() && self.manufacturer.is_none() {
          match self.namespace.cmp(&other.namespace) {
            std::cmp::Ordering::Equal => {}
            ord => return ord,
          }

          self.name.cmp(&other.name)
        } else {
          core::cmp::Ordering::Equal
        }
      }
      ord => ord,
    }
  }
}

impl ConnectionCounter {
  #[inline]
  pub const fn increment_active(&mut self) {
    self.active += 1;
  }

  #[inline]
  pub const fn increment_degraded(&mut self) {
    self.degraded += 1;
  }

  #[inline]
  pub const fn increment_failed(&mut self) {
    self.failed += 1;
  }

  pub const fn as_semantic_type(&self) -> SemanticType {
    if self.active > 0 {
      SemanticType::Success
    } else if self.degraded > 0 {
      SemanticType::Warning
    } else if self.failed > 0 {
      SemanticType::Error
    } else {
      SemanticType::Info
    }
  }
}

impl From<Arc<DeviceId>> for Link {
  #[inline]
  fn from(value: Arc<DeviceId>) -> Self {
    Self::Device(value)
  }
}

impl From<IpAddr> for Link {
  #[inline]
  fn from(value: IpAddr) -> Self {
    Self::Client(value)
  }
}

impl From<Arc<str>> for Link {
  #[inline]
  fn from(value: Arc<str>) -> Self {
    Self::Server(value)
  }
}

impl GraphNode for NutServerNode {
  fn get_semantic_type(&self) -> SemanticType {
    if self.status == ConnectionStatus::Dead {
      SemanticType::Error
    } else {
      self.connection_counter.as_semantic_type()
    }
  }

  #[inline]
  fn get_id(&self) -> NodeId {
    NodeId {
      id: self.id,
      node_type: NodeType::Server,
    }
  }
}

impl GraphNode for EdgeNode {
  #[inline]
  fn get_semantic_type(&self) -> SemanticType {
    self.semantic_type
  }

  #[inline]
  fn get_id(&self) -> NodeId {
    NodeId {
      id: self.id,
      node_type: NodeType::Edge,
    }
  }
}

impl GraphNode for ClientNode {
  #[inline]
  fn get_semantic_type(&self) -> SemanticType {
    self.connection_counter.as_semantic_type()
  }

  #[inline]
  fn get_id(&self) -> NodeId {
    NodeId {
      id: self.id,
      node_type: NodeType::Client,
    }
  }
}

impl GraphNode for DeviceNode {
  #[inline]
  fn get_semantic_type(&self) -> SemanticType {
    self.semantic_type
  }

  #[inline]
  fn get_id(&self) -> NodeId {
    NodeId {
      id: self.id,
      node_type: NodeType::Device,
    }
  }
}

impl core::fmt::Display for NodeId {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.write_fmt(format_args!(
      "{node_type}_{id}",
      node_type = self.node_type.as_str(),
      id = self.id
    ))
  }
}

impl askama::FastWritable for NodeId {
  fn write_into<W: core::fmt::Write + ?Sized>(
    &self,
    dest: &mut W,
    _: &dyn askama::Values,
  ) -> askama::Result<()> {
    dest.write_fmt(format_args!(
      "{node_type}_{id}",
      node_type = self.node_type.as_str(),
      id = self.id
    ))?;

    Ok(())
  }
}
