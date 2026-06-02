use super::known_metric::{KnownMetricDescriptors, METRIC_UPS_STATUS, METRIC_UPS_STATUS_HELP};
use crate::state::UpsdState;
use prometheus_client::{
  collector::Collector,
  encoding::{self, EncodeLabelSet},
  metrics::MetricType,
};
use std::{rc::Rc, sync::Arc};

pub struct UpsdStatCollector {
  inner: Arc<UpsdState>,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelSet)]
struct UpsdLabelSet {
  namespace: Arc<str>,
  device: Box<str>,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelSet)]
struct UpsStatusLabelSet {
  status: String,
  namespace: Arc<str>,
  device: Rc<str>,
}

impl UpsdStatCollector {
  #[inline]
  pub const fn new(upsd_state: Arc<UpsdState>) -> Self {
    Self { inner: upsd_state }
  }
}

impl Collector for UpsdStatCollector {
  fn encode(&self, mut encoder: encoding::DescriptorEncoder) -> Result<(), std::fmt::Error> {
    let state = self.inner.daemon_state.blocking_read();

    for (device_name, entry) in state.devices.iter() {
      let mut status_encoder = encoder.encode_descriptor(
        METRIC_UPS_STATUS,
        METRIC_UPS_STATUS_HELP,
        None,
        MetricType::Gauge,
      )?;
      let name: Rc<str> = Rc::from(device_name.as_str()); // current EncodeLabelSet does not allow references with lifetime

      for flag in entry.status.iter() {
        let status_label = UpsStatusLabelSet {
          status: flag.to_string(),
          namespace: self.inner.namespace.clone(),
          device: name.clone(),
        };

        status_encoder
          .encode_family(&status_label)?
          .encode_gauge(&1)?;
      }

      for flag in (!entry.status).iter() {
        let status_label = UpsStatusLabelSet {
          status: flag.to_string(),
          namespace: self.inner.namespace.clone(),
          device: name.clone(),
        };

        status_encoder
          .encode_family(&status_label)?
          .encode_gauge(&0)?;
      }

      for (var_name, value) in entry.variables.iter() {
        let metric_value = match value.as_lossy_f64() {
          Some(v) => v,
          None => continue,
        };

        if let Some(descriptor) = KnownMetricDescriptors::from_var_name(var_name) {
          let mut metric_encoder = encoder.encode_descriptor(
            descriptor.name,
            descriptor.help,
            descriptor.unit.as_ref(),
            descriptor.metric_type,
          )?;

          metric_encoder
            .encode_family(&UpsdLabelSet {
              namespace: self.inner.namespace.clone(),
              device: device_name.clone().into_boxed_str(),
            })?
            .encode_gauge(&metric_value)?;
        }
      }
    }

    Ok(())
  }
}

impl std::fmt::Debug for UpsdStatCollector {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("UpsdStatCollector").finish()
  }
}
