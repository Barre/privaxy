use lazy_static::lazy_static;
use std::sync::{Arc, RwLock};
use wildmatch::WildMatch;

#[derive(Debug, Clone)]
struct WildMatchCollection(Vec<WildMatch>);

impl WildMatchCollection {
    fn new(patterns: Vec<String>) -> Self {
        Self(
            patterns
                .into_iter()
                .map(|pattern| {
                    // Making things case insensitive

                    let pattern_lowercase = pattern.to_lowercase();
                    WildMatch::new(&pattern_lowercase)
                })
                .collect(),
        )
    }

    fn is_match(&self, element: &str) -> bool {
        // Making things case insensitive
        let lowercase_element = element.to_lowercase();

        self.0
            .iter()
            .any(|pattern| pattern.matches(&lowercase_element))
    }
}

lazy_static! {
    // We don't yet support defining globs for user defined exclusions as we rely on the glob crate, which
    // is designed to work on paths, not on hostnames.
    static ref DEFAULT_EXCLUSIONS: WildMatchCollection = {
        let mut exclusions = Vec::new();

        // Apple service exclusions, as defined in : https://support.apple.com/en-us/HT210060
        // > Apple services will fail any connection that uses
        // > HTTPS Interception (SSL Inspection). If the HTTPS traffic
        // > traverses a web proxy, disable HTTPS Interception for the hosts
        // > listed in this article.
        exclusions.push(String::from("*.apple.com"));
        exclusions.push(String::from("static.ips.apple.com"));
        exclusions.push(String::from("*.push.apple.com"));
        exclusions.push(String::from("setup.icloud.com"));
        exclusions.push(String::from("*.business.apple.com"));
        exclusions.push(String::from("*.school.apple.com"));
        exclusions.push(String::from("upload.appleschoolcontent.com"));
        exclusions.push(String::from("ws-ee-maidsvc.icloud.com"));
        exclusions.push(String::from("itunes.com"));
        exclusions.push(String::from("appldnld.apple.com.edgesuite.net"));
        exclusions.push(String::from("*.itunes.apple.com"));
        exclusions.push(String::from("updates-http.cdn-apple.com"));
        exclusions.push(String::from("updates.cdn-apple.com"));
        exclusions.push(String::from("*.apps.apple.com"));
        exclusions.push(String::from("*.mzstatic.com"));
        exclusions.push(String::from("*.appattest.apple.com"));
        exclusions.push(String::from("doh.dns.apple.com"));
        exclusions.push(String::from("appleid.cdn-apple.com"));
        exclusions.push(String::from("*.apple-cloudkit.com"));
        exclusions.push(String::from("*.apple-livephotoskit.com"));
        exclusions.push(String::from("*.apzones.com"));
        exclusions.push(String::from("*.cdn-apple.com"));
        exclusions.push(String::from("*.gc.apple.com"));
        exclusions.push(String::from("*.icloud.com"));
        exclusions.push(String::from("*.icloud.com.cn"));
        exclusions.push(String::from("*.icloud.apple.com"));
        exclusions.push(String::from("*.icloud-content.com"));
        exclusions.push(String::from("*.iwork.apple.com"));
        exclusions.push(String::from("mask.icloud.com"));
        exclusions.push(String::from("mask-h2.icloud.com"));
        exclusions.push(String::from("mask-api.icloud.com"));
        exclusions.push(String::from("devimages-cdn.apple.com"));
        exclusions.push(String::from("download.developer.apple.com"));

        WildMatchCollection::new(exclusions)
    };
}

#[derive(Debug, Clone)]
pub struct LocalExclusionStore(Arc<RwLock<WildMatchCollection>>);

impl LocalExclusionStore {
    pub fn new(exclusions: Vec<String>) -> Self {
        let collection = WildMatchCollection::new(exclusions);
        Self(Arc::new(RwLock::new(collection)))
    }

    pub fn replace_exclusions(&mut self, exclusions: Vec<String>) {
        let new_exclusion_store = LocalExclusionStore::new(exclusions);

        *self.0.write().unwrap() = new_exclusion_store.0.read().unwrap().clone();
    }

    pub fn contains(&self, element: &str) -> bool {
        // Items are stored lowercased
        let element_lowercase = element.to_lowercase();

        if DEFAULT_EXCLUSIONS.is_match(element) {
            true
        } else {
            self.0.read().unwrap().is_match(&element_lowercase)
        }
    }
}
