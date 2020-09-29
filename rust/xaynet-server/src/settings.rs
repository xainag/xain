//! Loading and validation of settings.
//!
//! Values defined in the configuration file can be overridden by environment variables. Examples of
//! configuration files can be found in the `configs/` directory located in the repository root.

use std::{fmt, path::PathBuf};

use config::{Config, ConfigError, Environment};
use redis::{ConnectionInfo, IntoConnectionInfo};
use serde::de::{self, Deserializer, Visitor};
use thiserror::Error;
use tracing_subscriber::filter::EnvFilter;
use validator::{Validate, ValidationError, ValidationErrors};

use xaynet_core::mask::{BoundType, DataType, GroupType, MaskConfig, ModelType};

#[derive(Error, Debug)]
/// An error related to loading and validation of settings.
pub enum SettingsError {
    #[error("configuration loading failed: {0}")]
    Loading(#[from] ConfigError),
    #[error("validation failed: {0}")]
    Validation(#[from] ValidationErrors),
}

#[derive(Debug, Validate, Deserialize)]
/// The combined settings.
///
/// Each section in the configuration file corresponds to the identically named settings field.
pub struct Settings {
    #[validate]
    pub api: ApiSettings,
    #[validate]
    pub pet: PetSettings,
    pub mask: MaskSettings,
    pub log: LoggingSettings,
    pub model: ModelSettings,
    #[validate]
    pub metrics: MetricsSettings,
    pub redis: RedisSettings,
}

impl Settings {
    /// Loads and validates the settings via a configuration file.
    ///
    /// # Errors
    /// Fails when the loading of the configuration file or its validation failed.
    pub fn new(path: PathBuf) -> Result<Self, SettingsError> {
        let settings: Settings = Self::load(path)?;
        settings.validate()?;
        Ok(settings)
    }

    fn load(path: PathBuf) -> Result<Self, ConfigError> {
        let mut config = Config::new();
        config.merge(config::File::from(path))?;
        config.merge(Environment::with_prefix("xaynet").separator("__"))?;
        config.try_into()
    }
}

#[derive(Debug, Validate, Deserialize, Clone, Copy)]
#[validate(schema(function = "validate_pet"))]
/// PET protocol settings.
pub struct PetSettings {
    #[validate(range(min = 1))]
    /// The minimal number of participants selected for computing the unmasking sum. The value must
    /// be greater or equal to `1` (i.e. `min_sum_count >= 1`), otherwise the PET protocol will be
    /// broken.
    ///
    /// This parameter should only be used to enforce security constraints.
    ///
    /// # Examples
    ///
    /// **TOML**
    /// ```text
    /// [pet]
    /// min_sum_count = 1
    /// ```
    ///
    /// **Environment variable**
    /// ```text
    /// XAYNET_PET__MIN_SUM_COUNT=1
    /// ```
    pub min_sum_count: usize,

    #[validate(range(min = 3))]
    /// The expected fraction of participants selected for submitting an updated local model for
    /// aggregation. The value must be greater or equal to `3` (i.e. `min_update_count >= 3`),
    /// otherwise the PET protocol will be broken.
    ///
    /// This parameter should only be used to enforce security constraints.
    ///
    /// # Examples
    ///
    /// **TOML**
    /// ```text
    /// [pet]
    /// min_update_count = 3
    /// ```
    ///
    /// **Environment variable**
    /// ```text
    /// XAYNET_PET__MIN_UPDATE_COUNT=3
    /// ```
    pub min_update_count: usize,

    /// The minimum amount of time reserved for processing messages in the `sum`
    /// and `sum2` phases, in seconds.
    ///
    /// Defaults to 0 i.e. `sum` and `sum2` phases end *as soon as*
    /// [`PetSettings::min_sum_count`] messages have been processed. Set this higher
    /// to allow for the possibility of more than
    /// [`PetSettings::min_sum_count`] messages to be processed in the
    /// `sum` and `sum2` phases.
    ///
    /// # Examples
    ///
    /// **TOML**
    /// ```text
    /// [pet]
    /// min_sum_time = 5
    /// ```
    ///
    /// **Environment variable**
    /// ```text
    /// XAYNET_PET__MIN_SUM_TIME=5
    /// ```
    pub min_sum_time: u64,

    /// The minimum amount of time reserved for processing messages in the
    /// `update` phase, in seconds.
    ///
    /// Defaults to 0 i.e. `update` phase ends *as soon as*
    /// [`PetSettings::min_update_count`] messages have been
    /// processed. Set this higher to allow for the possibility of
    /// more than [`PetSettings::min_update_count`] messages to be
    /// processed in the `update` phase.
    ///
    /// # Examples
    ///
    /// **TOML**
    /// ```text
    /// [pet]
    /// min_update_time = 10
    /// ```
    ///
    /// **Environment variable**
    /// ```text
    /// XAYNET_PET__MIN_UPDATE_TIME=10
    /// ```
    pub min_update_time: u64,

    /// The maximum amount of time permitted for processing messages in the `sum`
    /// and `sum2` phases, in seconds.
    ///
    /// Defaults to a large number (effectively 1 week). Set this
    /// lower to allow for the processing of
    /// [`PetSettings::min_sum_count`] messages to time-out sooner in
    /// the `sum` and `sum2` phases.
    ///
    /// # Examples
    ///
    /// **TOML**
    /// ```text
    /// [pet]
    /// max_sum_time = 30
    /// ```
    ///
    /// **Environment variable**
    /// ```text
    /// XAYNET_PET__MAX_SUM_TIME=30
    /// ```
    pub max_sum_time: u64,

    /// The maximum amount of time permitted for processing messages in the
    /// `update` phase, in seconds.
    ///
    /// Defaults to a large number (effectively 1 week). Set this
    /// lower to allow for the processing of
    /// [`PetSettings::min_update_count`] messages to time-out sooner
    /// in the `update` phase.
    ///
    /// # Examples
    ///
    /// **TOML**
    /// ```text
    /// [pet]
    /// max_update_time = 60
    /// ```
    ///
    /// **Environment variable**
    /// ```text
    /// XAYNET_PET__MAX_UPDATE_TIME=60
    /// ```
    pub max_update_time: u64,

    /// The expected fraction of participants selected for computing the unmasking sum. The value
    /// must be between `0` and `1` (i.e. `0 < sum < 1`).
    ///
    /// Additionally, it is enforced that `0 < sum + update - sum*update < 1` to avoid pathological
    /// cases of deadlocks.
    ///
    /// # Examples
    ///
    /// **TOML**
    /// ```text
    /// [pet]
    /// sum = 0.01
    /// ```
    ///
    /// **Environment variable**
    /// ```text
    /// XAYNET_PET__SUM=0.01
    /// ```
    pub sum: f64,

    /// The expected fraction of participants selected for submitting an updated local model for
    /// aggregation. The value must be between `0` and `1` (i.e. `0 < update < 1`).
    ///
    /// Additionally, it is enforced that `0 < sum + update - sum*update < 1` to avoid pathological
    /// cases of deadlocks.
    ///
    /// # Examples
    ///
    /// **TOML**
    /// ```text
    /// [pet]
    /// update = 0.01
    /// ```
    ///
    /// **Environment variable**
    /// ```text
    /// XAYNET_PET__UPDATE=0.01
    /// ```
    pub update: f64,
}

impl Default for PetSettings {
    fn default() -> Self {
        Self {
            min_sum_count: 1_usize,
            min_update_count: 3_usize,
            min_sum_time: 0_u64,
            min_update_time: 0_u64,
            max_sum_time: 604800_u64,
            max_update_time: 604800_u64,
            sum: 0.01_f64,
            update: 0.1_f64,
        }
    }
}

/// Checks PET settings.
fn validate_pet(s: &PetSettings) -> Result<(), ValidationError> {
    validate_phase_times(s)?;
    validate_fractions(s)
}

/// Checks validity of phase time ranges.
fn validate_phase_times(s: &PetSettings) -> Result<(), ValidationError> {
    if s.min_sum_time <= s.max_sum_time && s.min_update_time <= s.max_update_time {
        Ok(())
    } else {
        Err(ValidationError::new("invalid phase time range(s)"))
    }
}

/// Checks pathological cases of deadlocks.
fn validate_fractions(s: &PetSettings) -> Result<(), ValidationError> {
    if 0. < s.sum
        && s.sum < 1.
        && 0. < s.update
        && s.update < 1.
        && 0. < s.sum + s.update - s.sum * s.update
        && s.sum + s.update - s.sum * s.update < 1.
    {
        Ok(())
    } else {
        Err(ValidationError::new("starvation"))
    }
}

#[derive(Debug, Validate, Deserialize, Clone)]
/// REST API settings.
pub struct ApiSettings {
    /// The address to which the REST API should be bound.
    ///
    /// # Examples
    ///
    /// **TOML**
    /// ```text
    /// [api]
    /// bind_address = "0.0.0.0:8081"
    /// # or
    /// bind_address = "127.0.0.1:8081"
    /// ```
    ///
    /// **Environment variable**
    /// ```text
    /// XAYNET_API__BIND_ADDRESS=127.0.0.1:8081
    /// ```
    pub bind_address: std::net::SocketAddr,

    #[cfg(feature = "tls")]
    /// The path to the server certificate to enable TLS. If this is present, then `tls_key` must
    /// also be present.
    ///
    /// Requires the `tls` feature to be enabled.
    ///
    /// # Examples
    ///
    /// **TOML**
    /// ```text
    /// [api]
    /// tls_certificate = path/to/tls/files/cert.pem
    /// ```
    ///
    /// **Environment variable**
    /// ```text
    /// XAYNET_API__TLS_CERTIFICATE=path/to/tls/files/certificate.pem
    /// ```
    pub tls_certificate: String,

    #[cfg(feature = "tls")]
    /// The path to the server private key to enable TLS. If this is present, then `tls_certificate
    /// ` must also be present.
    ///
    /// Requires the `tls` feature to be enabled.
    ///
    /// # Examples
    ///
    /// **TOML**
    /// ```text
    /// [api]
    /// tls_key = path/to/tls/files/key.rsa
    /// ```
    ///
    /// **Environment variable**
    /// ```text
    /// XAYNET_API__TLS_KEY=path/to/tls/files/key.rsa
    /// ```
    pub tls_key: String,
}

#[derive(Debug, Validate, Deserialize, Clone, Copy)]
/// Masking settings.
pub struct MaskSettings {
    /// The order of the finite group.
    ///
    /// # Examples
    ///
    /// **TOML**
    /// ```text
    /// [mask]
    /// group_type = "Integer"
    /// ```
    ///
    /// **Environment variable**
    /// ```text
    /// XAYNET_MASK__GROUP_TYPE=Integer
    /// ```
    pub group_type: GroupType,

    /// The data type of the numbers to be masked.
    ///
    /// # Examples
    ///
    /// **TOML**
    /// ```text
    /// [mask]
    /// data_type = "F32"
    /// ```
    ///
    /// **Environment variable**
    /// ```text
    /// XAYNET_MASK__DATA_TYPE=F32
    /// ```
    pub data_type: DataType,

    /// The bounds of the numbers to be masked.
    ///
    /// # Examples
    ///
    /// **TOML**
    /// ```text
    /// [mask]
    /// bound_type = "B0"
    /// ```
    ///
    /// **Environment variable**
    /// ```text
    /// XAYNET_MASK__BOUND_TYPE=B0
    /// ```
    pub bound_type: BoundType,

    /// The maximum number of models to be aggregated.
    ///
    /// # Examples
    ///
    /// **TOML**
    /// ```text
    /// [mask]
    /// model_type = "M3"
    /// ```
    ///
    /// **Environment variable**
    /// ```text
    /// XAYNET_MASK__MODEL_TYPE=M3
    /// ```
    pub model_type: ModelType,
}

impl Default for MaskSettings {
    fn default() -> Self {
        Self {
            group_type: GroupType::Prime,
            data_type: DataType::F32,
            bound_type: BoundType::B0,
            model_type: ModelType::M3,
        }
    }
}

impl From<MaskSettings> for MaskConfig {
    fn from(
        MaskSettings {
            group_type,
            data_type,
            bound_type,
            model_type,
        }: MaskSettings,
    ) -> MaskConfig {
        MaskConfig {
            group_type,
            data_type,
            bound_type,
            model_type,
        }
    }
}

#[derive(Debug, Deserialize)]
/// Model settings.
pub struct ModelSettings {
    /// The expected size of the model. The model size corresponds to the number of elements.
    /// This value is used to validate the uniform length of the submitted models/masks.
    ///
    /// # Examples
    ///
    /// **TOML**
    /// ```text
    /// [model]
    /// size = 100
    /// ```
    ///
    /// **Environment variable**
    /// ```text
    /// XAYNET_MODEL__SIZE=100
    /// ```
    pub size: usize,
}

#[derive(Debug, Deserialize, Validate)]
/// Metrics settings.
pub struct MetricsSettings {
    #[validate]
    /// Settings for the InfluxDB backend.
    pub influxdb: InfluxSettings,
}

#[derive(Debug, Deserialize, Validate)]
/// InfluxDB settings.
pub struct InfluxSettings {
    #[validate(url)]
    /// The URL where InfluxDB is running.
    ///
    /// # Examples
    ///
    /// **TOML**
    /// ```text
    /// [metrics.influxdb]
    /// url = "http://localhost:8086"
    /// ```
    ///
    /// **Environment variable**
    /// ```text
    /// XAYNET_METRICS__INFLUXDB__URL=http://localhost:8086
    /// ```
    pub url: String,

    /// The InfluxDB database name.
    ///
    /// # Examples
    ///
    /// **TOML**
    /// ```text
    /// [metrics.influxdb]
    /// db = "test"
    /// ```
    ///
    /// **Environment variable**
    /// ```text
    /// XAYNET_METRICS__INFLUXDB__DB=test
    /// ```
    pub db: String,
}

#[derive(Debug, Deserialize)]
/// Redis settings.
pub struct RedisSettings {
    /// The URL where Redis is running.
    ///
    /// The format of the URL is `redis://[<username>][:<passwd>@]<hostname>[:port][/<db>]`.
    ///
    /// # Examples
    ///
    /// **TOML**
    /// ```text
    /// [redis]
    /// url = "redis://127.0.0.1/"
    /// ```
    ///
    /// **Environment variable**
    /// ```text
    /// XAYNET_REDIS__URL=redis://127.0.0.1/
    /// ```
    #[serde(deserialize_with = "deserialize_redis_url")]
    pub url: ConnectionInfo,
}

fn deserialize_redis_url<'de, D>(deserializer: D) -> Result<ConnectionInfo, D::Error>
where
    D: Deserializer<'de>,
{
    struct ConnectionInfoVisitor;

    impl<'de> Visitor<'de> for ConnectionInfoVisitor {
        type Value = ConnectionInfo;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            write!(
                formatter,
                "redis://[<username>][:<passwd>@]<hostname>[:port][/<db>]"
            )
        }

        fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            value
                .into_connection_info()
                .map_err(|_| de::Error::invalid_value(serde::de::Unexpected::Str(value), &self))
        }
    }

    deserializer.deserialize_str(ConnectionInfoVisitor)
}

#[derive(Debug, Deserialize)]
/// Logging settings.
pub struct LoggingSettings {
    /// A comma-separated list of logging directives. More information about logging directives
    /// can be found [here].
    ///
    /// # Examples
    ///
    /// **TOML**
    /// ```text
    /// [log]
    /// filter = "info"
    /// ```
    ///
    /// **Environment variable**
    /// ```text
    /// XAYNET_LOG__FILTER=info
    /// ```
    ///
    /// [here]: https://docs.rs/tracing-subscriber/0.2.6/tracing_subscriber/filter/struct.EnvFilter.html#directives
    #[serde(deserialize_with = "deserialize_env_filter")]
    pub filter: EnvFilter,
}

fn deserialize_env_filter<'de, D>(deserializer: D) -> Result<EnvFilter, D::Error>
where
    D: Deserializer<'de>,
{
    struct EnvFilterVisitor;

    impl<'de> Visitor<'de> for EnvFilterVisitor {
        type Value = EnvFilter;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            write!(formatter, "a valid tracing filter directive: https://docs.rs/tracing-subscriber/0.2.6/tracing_subscriber/filter/struct.EnvFilter.html#directives")
        }

        fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            EnvFilter::try_new(value)
                .map_err(|_| de::Error::invalid_value(serde::de::Unexpected::Str(value), &self))
        }
    }

    deserializer.deserialize_str(EnvFilterVisitor)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(not(feature = "tls"))]
    #[test]
    fn test_settings_new() {
        assert!(Settings::new(PathBuf::from("../../configs/config.toml")).is_ok());
        assert!(Settings::new(PathBuf::from("")).is_err());
    }

    #[test]
    fn test_validate_pet() {
        assert!(validate_pet(&PetSettings::default()).is_ok());

        // phase times
        assert!(validate_pet(&PetSettings {
            min_sum_time: 2,
            max_sum_time: 1,
            ..PetSettings::default()
        })
        .is_err());
        assert!(validate_pet(&PetSettings {
            min_update_time: 2,
            max_update_time: 1,
            ..PetSettings::default()
        })
        .is_err());

        // fractions
        assert!(validate_pet(&PetSettings {
            sum: 0.,
            ..PetSettings::default()
        })
        .is_err());
        assert!(validate_pet(&PetSettings {
            sum: 1.,
            ..PetSettings::default()
        })
        .is_err());
        assert!(validate_pet(&PetSettings {
            update: 0.,
            ..PetSettings::default()
        })
        .is_err());
        assert!(validate_pet(&PetSettings {
            update: 1.,
            ..PetSettings::default()
        })
        .is_err());
    }
}
