use crate::{
    blocker::AdblockRequester, ca::make_ca_certificate, proxy::exclusions::LocalExclusionStore,
};
use dirs::home_dir;
use futures::future::{try_join_all, AbortHandle, Abortable};
use openssl::{
    pkey::{PKey, Private},
    x509::X509,
};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeSet, time::Duration};
use std::{collections::HashSet, path::PathBuf};
use thiserror::Error;
use tokio::sync::{self, mpsc::Sender};
use tokio::{fs, sync::mpsc::Receiver};
use url::Url;

const BASE_FILTERS_URL: &str = "https://filters.privaxy.net";
const METADATA_FILE_NAME: &str = "metadata.json";
const CONFIGURATION_DIRECTORY_NAME: &str = ".privaxy";
const CONFIGURATION_FILE_NAME: &str = "config";
const FILTERS_DIRECTORY_NAME: &str = "filters";

// Update filters every hour.
const FILTERS_UPDATE_AFTER: Duration = Duration::from_secs(60 * 60);

type ConfigurationResult<T> = Result<T, ConfigurationError>;

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
enum FilterGroup {
    Default,
    Regional,
    Ads,
    Privacy,
    Malware,
    Social,
}

#[derive(Deserialize)]
pub struct DefaultFilter {
    enabled_by_default: bool,
    file_name: String,
    group: String,
    title: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct Filter {
    enabled: bool,
    title: String,
    group: FilterGroup,
    file_name: String,
}

impl Filter {
    async fn update(&self, http_client: &reqwest::Client) -> ConfigurationResult<String> {
        log::debug!("Updating filter: {}", self.title);

        let home_directory = get_home_directory()?;
        let configuration_directory = home_directory.join(CONFIGURATION_DIRECTORY_NAME);
        let filters_directory = configuration_directory.join(FILTERS_DIRECTORY_NAME);

        fs::create_dir_all(&filters_directory).await?;

        let filter = get_filter(&self.file_name, http_client).await?;

        fs::write(filters_directory.join(&self.file_name), &filter).await?;

        Ok(filter)
    }

    pub async fn get_contents(&self, http_client: &reqwest::Client) -> ConfigurationResult<String> {
        let filter_path = get_home_directory()?
            .join(CONFIGURATION_DIRECTORY_NAME)
            .join(FILTERS_DIRECTORY_NAME)
            .join(&self.file_name);

        match fs::read(filter_path).await {
            Err(err) => {
                if err.kind() == std::io::ErrorKind::NotFound {
                    self.update(http_client).await
                } else {
                    Err(ConfigurationError::FileSystemError(err))
                }
            }
            Ok(filter) => Ok(std::str::from_utf8(&filter)?.to_string()),
        }
    }
}

impl From<DefaultFilter> for Filter {
    fn from(default_filter: DefaultFilter) -> Self {
        Self {
            enabled: default_filter.enabled_by_default,
            title: default_filter.title,
            group: match default_filter.group.as_str() {
                "default" => FilterGroup::Default,
                "regional" => FilterGroup::Regional,
                "ads" => FilterGroup::Ads,
                "privacy" => FilterGroup::Privacy,
                "malware" => FilterGroup::Malware,
                "social" => FilterGroup::Social,
                _ => unreachable!(),
            },
            file_name: default_filter.file_name,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct Ca {
    ca_certificate: String,
    ca_private_key: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct Configuration {
    pub exclusions: BTreeSet<String>,
    pub custom_filters: Vec<String>,
    ca: Ca,
    pub filters: Vec<Filter>,
}

#[derive(Error, Debug)]
pub enum ConfigurationError {
    #[error("an error occured while trying to deserialize configuration file")]
    DeserializeError(#[from] toml::de::Error),
    #[error("this user home directory not found")]
    HomeDirectoryNotFound,
    #[error("file system error")]
    FileSystemError(#[from] std::io::Error),
    #[error("data store disconnected")]
    UnableToRetrieveDefaultFilters(#[from] reqwest::Error),
    #[error("unable to decode filter bytes, bad utf8 data")]
    UnableToDecodeFilterbytes(#[from] std::str::Utf8Error),
    #[error("unable to decode pem data")]
    UnableToDecodePem(#[from] openssl::error::ErrorStack),
}

impl Configuration {
    pub async fn read_from_home(http_client: reqwest::Client) -> ConfigurationResult<Self> {
        let home_directory = get_home_directory()?;
        let configuration_directory = home_directory.join(CONFIGURATION_DIRECTORY_NAME);
        let configuration_file_path = configuration_directory.join(CONFIGURATION_FILE_NAME);

        if let Err(err) = fs::metadata(&configuration_directory).await {
            if err.kind() == std::io::ErrorKind::NotFound {
                log::debug!("Configuration directory not found, creating one");

                fs::create_dir(&configuration_directory).await?;

                let configuration = Self::new_default(http_client).await?;
                configuration.save().await?;

                return Ok(configuration);
            } else {
                return Err(ConfigurationError::FileSystemError(err));
            }
        };

        match fs::read(&configuration_file_path).await {
            Ok(bytes) => Ok(toml::from_slice(&bytes)?),
            Err(err) => {
                log::debug!("Configuration file not found, creating one");

                if err.kind() == std::io::ErrorKind::NotFound {
                    let configuration = Self::new_default(http_client).await?;
                    configuration.save().await?;

                    Ok(configuration)
                } else {
                    Err(ConfigurationError::FileSystemError(err))
                }
            }
        }
    }

    pub async fn save(&self) -> ConfigurationResult<()> {
        let home_directory = get_home_directory()?;
        let configuration_directory = home_directory.join(CONFIGURATION_DIRECTORY_NAME);
        let configuration_file_path = configuration_directory.join(CONFIGURATION_FILE_NAME);

        let configuration_serialized = toml::to_string_pretty(&self).unwrap();

        fs::write(configuration_file_path, configuration_serialized).await?;

        Ok(())
    }

    pub async fn set_custom_filters(&mut self, custom_filters: &str) -> ConfigurationResult<()> {
        self.custom_filters = Self::deserialize_lines(custom_filters);

        self.save().await?;

        Ok(())
    }

    fn deserialize_lines<T>(lines: &str) -> T
    where
        T: FromIterator<String>,
    {
        lines
            .lines()
            .filter_map(|s_| {
                let s_ = s_.trim();

                // Removing empty lines
                if s_.is_empty() {
                    None
                } else {
                    Some(s_.to_string())
                }
            })
            .collect::<T>()
    }

    pub async fn set_exclusions(
        &mut self,
        exclusions: &str,
        mut local_exclusion_store: LocalExclusionStore,
    ) -> ConfigurationResult<()> {
        self.exclusions = Self::deserialize_lines(exclusions);

        self.save().await?;

        local_exclusion_store
            .replace_exclusions(HashSet::from_iter(self.exclusions.clone().into_iter()));

        Ok(())
    }

    pub async fn set_filter_enabled_status(
        &mut self,
        filter_file_name: &str,
        enabled: bool,
    ) -> ConfigurationResult<()> {
        let filter = self
            .filters
            .iter_mut()
            .find(|filter| filter.file_name == filter_file_name);

        if let Some(filter) = filter {
            filter.enabled = enabled;
        }

        self.save().await?;
        Ok(())
    }

    pub fn get_enabled_filters(&self) -> Vec<&Filter> {
        self.filters
            .iter()
            .filter(|filter| filter.enabled)
            .collect()
    }

    pub async fn update_filters(&self, http_client: reqwest::Client) -> ConfigurationResult<()> {
        log::debug!("Updating filters");

        let futures = self.filters.iter().filter_map(|filter| {
            if filter.enabled {
                Some(filter.update(&http_client))
            } else {
                None
            }
        });

        try_join_all(futures).await?;

        Ok(())
    }

    pub fn ca_certificate(&self) -> ConfigurationResult<X509> {
        Ok(X509::from_pem(self.ca.ca_certificate.as_bytes())?)
    }

    pub fn ca_private_key(&self) -> ConfigurationResult<PKey<Private>> {
        Ok(PKey::private_key_from_pem(
            self.ca.ca_private_key.as_bytes(),
        )?)
    }

    async fn new_default(http_client: reqwest::Client) -> ConfigurationResult<Self> {
        let default_filters = get_default_filters(http_client).await?;

        let (x509, private_key) = make_ca_certificate();

        let x509_pem = std::str::from_utf8(&x509.to_pem().unwrap())
            .unwrap()
            .to_string();

        let private_key_pem = std::str::from_utf8(&private_key.private_key_to_pem_pkcs8().unwrap())
            .unwrap()
            .to_string();

        Ok(Configuration {
            filters: default_filters
                .into_iter()
                .map(|filter| filter.into())
                .collect(),
            ca: Ca {
                ca_certificate: x509_pem,
                ca_private_key: private_key_pem,
            },
            exclusions: BTreeSet::new(),
            custom_filters: Vec::new(),
        })
    }
}

async fn get_default_filters(
    http_client: reqwest::Client,
) -> ConfigurationResult<Vec<DefaultFilter>> {
    let base_filters_url = BASE_FILTERS_URL.parse::<Url>().unwrap();
    let filters_url = base_filters_url.join(METADATA_FILE_NAME).unwrap();

    let response = http_client.get(filters_url.as_str()).send().await?;

    let default_filters = response.json::<Vec<DefaultFilter>>().await?;

    Ok(default_filters)
}

fn get_home_directory() -> ConfigurationResult<PathBuf> {
    match home_dir() {
        Some(home_directory) => Ok(home_directory),
        None => Err(ConfigurationError::HomeDirectoryNotFound),
    }
}

async fn get_filter(
    filter_file_name: &str,
    http_client: &reqwest::Client,
) -> ConfigurationResult<String> {
    let base_filters_url = BASE_FILTERS_URL.parse::<Url>().unwrap();
    let filter_url = base_filters_url.join(filter_file_name).unwrap();

    let response = http_client.get(filter_url.as_str()).send().await?;

    let filter = response.text().await?;

    Ok(filter)
}

pub struct ConfigurationUpdater {
    filters_updater_abort_handle: AbortHandle,
    rx: Receiver<Configuration>,
    pub tx: Sender<Configuration>,
    http_client: reqwest::Client,
    adblock_requester: AdblockRequester,
}

impl ConfigurationUpdater {
    pub(crate) async fn new(
        configuration: Configuration,
        http_client: reqwest::Client,
        adblock_requester: AdblockRequester,
        tx_rx: Option<(
            sync::mpsc::Sender<Configuration>,
            sync::mpsc::Receiver<Configuration>,
        )>,
    ) -> Self {
        let (abort_handle, abort_registration) = AbortHandle::new_pair();

        let (tx, rx) = match tx_rx {
            Some((tx, rx)) => (tx, rx),
            None => sync::mpsc::channel(1),
        };

        let http_client_clone = http_client.clone();
        let adblock_requester_clone = adblock_requester.clone();

        let filters_updater = Abortable::new(
            async move {
                Self::filters_updater(
                    configuration,
                    adblock_requester_clone,
                    http_client_clone.clone(),
                )
                .await
            },
            abort_registration,
        );

        tokio::spawn(filters_updater);

        Self {
            filters_updater_abort_handle: abort_handle,
            rx,
            tx,
            http_client,
            adblock_requester,
        }
    }

    pub(crate) fn start(mut self) {
        tokio::spawn(async move {
            if let Some(configuration) = self.rx.recv().await {
                self.filters_updater_abort_handle.abort();

                let filters = get_filters_content(&configuration, &self.http_client).await;

                self.adblock_requester.replace_engine(filters).await;

                let new_self = Self::new(
                    configuration,
                    self.http_client,
                    self.adblock_requester,
                    Some((self.tx, self.rx)),
                )
                .await;
                new_self.start();

                log::info!("Applied new configuration");
            }
        });
    }

    async fn filters_updater(
        configuration: Configuration,
        adblock_requester: AdblockRequester,
        http_client: reqwest::Client,
    ) {
        loop {
            tokio::time::sleep(FILTERS_UPDATE_AFTER).await;

            if let Err(err) = configuration.update_filters(http_client.clone()).await {
                log::error!("An error occured while trying to update filters: {:?}", err);
            }

            // We don't bother diffing the filters as replacing the engine is very cheap and
            // filters are not updated often enough that the cost would matter.
            let filters = get_filters_content(&configuration, &http_client).await;
            adblock_requester.replace_engine(filters).await;

            log::info!("Updated filters");
        }
    }
}

async fn get_filters_content(
    configuration: &Configuration,
    http_client: &reqwest::Client,
) -> Vec<String> {
    let mut filters = Vec::new();

    for filter in configuration.get_enabled_filters() {
        match filter.get_contents(http_client).await {
            Ok(filter) => filters.push(filter),
            Err(err) => {
                log::error!("Unable to retrieve filter: {:?}, skipping.", err)
            }
        }
    }

    filters.append(&mut configuration.custom_filters.clone());

    filters
}
