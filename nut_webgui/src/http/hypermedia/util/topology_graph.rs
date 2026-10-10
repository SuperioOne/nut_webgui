use super::named_enum::create_named_enum;
use crate::{
  http::hypermedia::util::{
    semantic_type::SemanticType,
    unit::{Percentage, RemainingSeconds, Voltage},
  },
  state::{ConnectionStatus, DaemonState, DeviceEntry, UpsdNamespace},
};
use std::{collections::BTreeMap, net::IpAddr, sync::Arc};
use upsmc::{UpsName, VarName, ups_status::UpsStatus};

const STATE_UNKOWN: UpsStatus = UpsStatus::NOCOMM
  .set(UpsStatus::OFFLINE)
  .set(UpsStatus::BYPASS);

const STATE_DEGRADED: UpsStatus = UpsStatus::ON_BATTERY
  .set(UpsStatus::TRIM)
  .set(UpsStatus::BOOST);

const STATE_CRITICAL: UpsStatus = UpsStatus::FORCED_SHUTDOWN
  .set(UpsStatus::LOW_BATTERY)
  .set(UpsStatus::REPLACE_BATTERY)
  .set(UpsStatus::ALARM)
  .set(UpsStatus::OVERLOADED);

pub trait GraphNode {
  fn get_semantic_type(&self) -> SemanticType;
  fn get_id(&self) -> NodeId;
}

pub struct TopologyGraph {
  pub clients: BTreeMap<IpAddr, ClientNode>,
  pub devices: BTreeMap<Arc<DeviceId>, DeviceNode>,
  pub edges: Vec<EdgeNode>,
  pub servers: BTreeMap<UpsdNamespace, NutServerNode>,
}

pub struct TopologyGraphBuilder {
  clients: BTreeMap<IpAddr, ClientNode>,
  devices: BTreeMap<Arc<DeviceId>, DeviceNode>,
  edges: Vec<EdgeNode>,
  servers: BTreeMap<UpsdNamespace, NutServerNode>,
}

#[derive(Eq, PartialEq)]
pub struct DeviceId {
  pub manufacturer: Option<String>,
  pub model: Option<String>,
  pub serial: Option<String>,
  pub name: UpsName,
  pub namespace: UpsdNamespace,
}

pub struct DeviceAlias {
  pub name: UpsName,
  pub namespace: Arc<str>,
}

create_named_enum!(
    #[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
    named_enum NodeType {
        Server = "server",
        Client = "client",
        Device = "device",
        Edge   = "edge"
    }
);

#[derive(Copy, Clone, PartialEq, PartialOrd, Eq, Ord)]
pub struct NodeId {
  id: usize,
  node_type: NodeType,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum EdgeType {
  Data,
  Power,
}

#[derive(Clone, Copy, PartialEq, Eq, Default, PartialOrd, Ord)]
enum PowerStatusLevel {
  /// No power indicator.
  #[default]
  Unknown = 0,
  /// All power sources are either dead or about to be dead.
  Critical = 1,
  /// At least one power source is available, but it's degraded.
  Degraded = 2,
  /// At least one healty power source is available.
  Active = 3,
}

pub struct ClientNode {
  power_status: PowerStatusLevel,
  id: usize,
  pub address: IpAddr,
  pub name: Option<Box<str>>,
}

pub struct NutServerNode {
  power_status: PowerStatusLevel,
  id: usize,
  pub address: Box<str>,
  pub namespace: Arc<str>,
  pub port: u16,
  pub status: ConnectionStatus,
}

pub struct DeviceNode {
  id: usize,
  power_status: PowerStatusLevel,
  pub aliases: Vec<DeviceAlias>,
  pub battery: Option<Percentage>,
  pub device_id: Arc<DeviceId>,
  pub input_voltage: Option<Voltage>,
  pub load: Option<Percentage>,
  pub runtime: Option<RemainingSeconds>,
  pub status: UpsStatus,
}

pub struct EdgeNode {
  semantic_type: SemanticType,
  id: usize,
  pub edge_type: EdgeType,
  pub from: NodeId,
  pub to: NodeId,
}

impl TopologyGraphBuilder {
  #[inline]
  pub const fn new() -> Self {
    Self {
      clients: BTreeMap::new(),
      devices: BTreeMap::new(),
      edges: Vec::new(),
      servers: BTreeMap::new(),
    }
  }

  pub fn add_upsd_server(
    &mut self,
    namespace: Arc<str>,
    address: Box<str>,
    port: u16,
    upsd_state: &DaemonState,
  ) {
    let mut server_node = NutServerNode {
      id: self.servers.len() + 1,
      address,
      namespace,
      port,
      status: upsd_state.status,
      power_status: PowerStatusLevel::default(),
    };

    for device in upsd_state.devices.values() {
      _ = self.add_device(device, &mut server_node);
    }

    _ = self
      .servers
      .insert(server_node.namespace.clone(), server_node);
  }

  #[inline]
  pub fn finish(self) -> TopologyGraph {
    TopologyGraph {
      devices: self.devices,
      edges: self.edges,
      clients: self.clients,
      servers: self.servers,
    }
  }

  fn add_device(&mut self, device: &DeviceEntry, server_node: &mut NutServerNode) {
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

    let device_node_id = match self.devices.get_mut(&key) {
      Some(node) => {
        node.aliases.push(DeviceAlias {
          name: device.name.to_owned(),
          namespace: server_node.namespace.clone(),
        });

        node.get_id()
      }
      None => {
        let key = Arc::new(key);
        let device_node = DeviceNode {
          id: self.devices.len(),
          device_id: key.clone(),
          aliases: vec![DeviceAlias {
            name: device.name.to_owned(),
            namespace: server_node.namespace.clone(),
          }],
          battery: device.get_battery_charge(),
          input_voltage: device.get_input_voltage(),
          load: device.get_ups_load(),
          runtime: device.get_battery_runtime(),
          power_status: PowerStatusLevel::from_ups_status(device.status),
          status: device.status,
        };

        let node_id = device_node.get_id();
        for client in device.attached.iter() {
          let target_id = if client.addr.is_loopback() {
            server_node
              .power_status
              .try_update(device_node.power_status);

            server_node.get_id()
          } else {
            self.add_client(client.addr, client.name.clone(), device_node.power_status)
          };

          self.add_edge(
            node_id,
            target_id,
            EdgeType::Power,
            device_node.power_status.into(),
          );
        }

        self.devices.insert(key, device_node);
        node_id
      }
    };

    let edge_semantic = if device.status.has(UpsStatus::NOCOMM) {
      SemanticType::Error
    } else {
      SemanticType::Info
    };

    self.add_edge(
      device_node_id,
      server_node.get_id(),
      EdgeType::Data,
      edge_semantic,
    );
  }

  fn add_client(
    &mut self,
    address: IpAddr,
    name: Option<Box<str>>,
    power_status: PowerStatusLevel,
  ) -> NodeId {
    let next_id = self.clients.len();
    let client = self.clients.entry(address).or_insert_with(|| ClientNode {
      id: next_id,
      address,
      name: name.clone(),
      power_status: PowerStatusLevel::default(),
    });

    client.power_status.try_update(power_status);
    client.get_id()
  }

  fn add_edge(
    &mut self,
    from: NodeId,
    to: NodeId,
    edge_type: EdgeType,
    semantic_type: SemanticType,
  ) {
    self.edges.push(EdgeNode {
      id: self.edges.len(),
      to,
      from,
      edge_type,
      semantic_type,
    });
  }
}

impl EdgeType {
  pub const fn as_class(&self) -> &'static str {
    match self {
      EdgeType::Power => "edge-dashed",
      EdgeType::Data => "edge-dotted",
    }
  }
}

impl PowerStatusLevel {
  #[inline]
  fn try_update(&mut self, value: Self) {
    if *self < value {
      *self = value;
    }
  }

  const fn from_ups_status(ups_status: UpsStatus) -> Self {
    if !ups_status.intersect(STATE_UNKOWN).is_empty() {
      PowerStatusLevel::Unknown
    } else if !ups_status.intersect(STATE_CRITICAL).is_empty() {
      PowerStatusLevel::Critical
    } else if !ups_status.intersect(STATE_DEGRADED).is_empty() {
      PowerStatusLevel::Degraded
    } else {
      PowerStatusLevel::Active
    }
  }
}

// This is a one-way conversion
impl Into<SemanticType> for PowerStatusLevel {
  fn into(self) -> SemanticType {
    match self {
      PowerStatusLevel::Unknown => SemanticType::None,
      PowerStatusLevel::Critical => SemanticType::Error,
      PowerStatusLevel::Degraded => SemanticType::Warning,
      PowerStatusLevel::Active => SemanticType::Success,
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
        if self.serial.is_none() {
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

impl GraphNode for NutServerNode {
  fn get_semantic_type(&self) -> SemanticType {
    if self.status == ConnectionStatus::Dead {
      SemanticType::Error
    } else {
      self.power_status.into()
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
    self.power_status.into()
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
    self.power_status.into()
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
  fn write_into(
    &self,
    dest: &mut dyn core::fmt::Write,
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
