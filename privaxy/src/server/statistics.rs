use serde::Serialize;
use std::{
    collections::HashMap,
    net::IpAddr,
    sync::{Arc, Mutex},
};
use uluru::LRUCache;

const ENTRIES_PER_STATISTICS_TABLE: u8 = 50;

#[derive(Debug, Serialize)]
pub(crate) struct SerializableStatistics {
    pub proxied_requests: u64,
    pub blocked_requests: u64,
    pub modified_responses: u64,
    #[serde(with = "tuple_vec_map")]
    pub top_blocked_paths: Vec<(String, u64)>,
    #[serde(with = "tuple_vec_map")]
    pub top_clients: Vec<(String, u64)>,
}

#[derive(Debug, Clone)]
pub(crate) struct Statistics {
    pub proxied_requests: Arc<Mutex<u64>>,
    pub blocked_requests: Arc<Mutex<u64>>,
    pub modified_responses: Arc<Mutex<u64>>,
    pub top_blocked_paths: Arc<Mutex<LRUCache<(String, u64), 1_000>>>,
    pub top_clients: Arc<Mutex<HashMap<IpAddr, u64>>>,
}

impl Statistics {
    pub fn new() -> Self {
        Self {
            proxied_requests: Arc::new(Mutex::new(0)),
            blocked_requests: Arc::new(Mutex::new(0)),
            modified_responses: Arc::new(Mutex::new(0)),
            top_blocked_paths: Arc::new(Mutex::new(LRUCache::default())),
            top_clients: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn increment_top_blocked_paths(&self, path_: String) {
        let mut top_blocked_paths = self.top_blocked_paths.lock().unwrap();

        match top_blocked_paths.find(|(path, _count)| path == &path_) {
            Some((_path, count)) => {
                *count += 1;
            }
            None => {
                top_blocked_paths.insert((path_, 1));
            }
        }
    }

    pub fn increment_top_clients(&self, client: IpAddr) {
        *self.top_clients.lock().unwrap().entry(client).or_insert(0) += 1;
    }

    pub fn increment_proxied_requests(&self) -> u64 {
        let mut proxied_requests = self.proxied_requests.lock().unwrap();

        *proxied_requests += 1;
        *proxied_requests
    }

    pub fn increment_blocked_requests(&self) -> u64 {
        let mut blocked_requests = self.blocked_requests.lock().unwrap();

        *blocked_requests += 1;
        *blocked_requests
    }

    pub fn increment_modified_responses(&self) -> u64 {
        let mut modified_responses = self.modified_responses.lock().unwrap();

        *modified_responses += 1;
        *modified_responses
    }

    pub fn get_serialized(&self) -> SerializableStatistics {
        SerializableStatistics {
            proxied_requests: *self.proxied_requests.lock().unwrap(),
            blocked_requests: *self.blocked_requests.lock().unwrap(),
            modified_responses: *self.modified_responses.lock().unwrap(),
            top_blocked_paths: {
                let top_blocked_paths = self.top_blocked_paths.lock().unwrap();
                let mut top_blocked_paths_iterator = top_blocked_paths.iter();

                let mut top_blocked_paths = (0..=ENTRIES_PER_STATISTICS_TABLE)
                    .into_iter()
                    .filter_map(|_| {
                        let (path, count) = top_blocked_paths_iterator.next()?;

                        Some((path.clone(), *count))
                    })
                    .collect::<Vec<_>>();

                top_blocked_paths.sort_by(|a, b| b.1.cmp(&a.1));

                top_blocked_paths
            },
            top_clients: {
                let top_clients = self.top_clients.lock().unwrap();
                let mut top_clients_iter = top_clients.iter();

                let mut top_clients = (0..=ENTRIES_PER_STATISTICS_TABLE)
                    .into_iter()
                    .filter_map(|_| {
                        let (ipv4, count) = top_clients_iter.next()?;

                        Some((ipv4.to_string(), *count))
                    })
                    .collect::<Vec<_>>();

                top_clients.sort_by(|a, b| b.1.cmp(&a.1));

                top_clients
            },
        }
    }
}
