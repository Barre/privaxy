use globset::{Glob, GlobSetBuilder};
use lazy_static::lazy_static;
use std::collections::HashSet;
use std::sync::{Arc, RwLock};

lazy_static! {
    // We don't yet support defining globs for user defined exclusions as we rely on the glob crate, which
    // is designed to work on paths, not on hostnames.
    static ref DEFAULT_EXCLUSIONS: globset::GlobSet = {
        let mut builder = GlobSetBuilder::new();
        // Apple service exclusions, as defined in : https://support.apple.com/en-us/HT210060
        // > Apple services will fail any connection that uses
        // > HTTPS Interception (SSL Inspection). If the HTTPS traffic
        // > traverses a web proxy, disable HTTPS Interception for the hosts
        // > listed in this article.
        builder.add(Glob::new("*.apple.com").unwrap());
        builder.add(Glob::new("static.ips.apple.com").unwrap());
        builder.add(Glob::new("*.push.apple.com").unwrap());
        builder.add(Glob::new("setup.icloud.com").unwrap());
        builder.add(Glob::new("*.business.apple.com").unwrap());
        builder.add(Glob::new("*.school.apple.com").unwrap());
        builder.add(Glob::new("upload.appleschoolcontent.com").unwrap());
        builder.add(Glob::new("ws-ee-maidsvc.icloud.com").unwrap());
        builder.add(Glob::new("itunes.com").unwrap());
        builder.add(Glob::new("appldnld.apple.com.edgesuite.net").unwrap());
        builder.add(Glob::new("*.itunes.apple.com").unwrap());
        builder.add(Glob::new("updates-http.cdn-apple.com").unwrap());
        builder.add(Glob::new("updates.cdn-apple.com").unwrap());
        builder.add(Glob::new("*.apps.apple.com").unwrap());
        builder.add(Glob::new("*.mzstatic.com").unwrap());
        builder.add(Glob::new("*.appattest.apple.com").unwrap());
        builder.add(Glob::new("doh.dns.apple.com").unwrap());
        builder.add(Glob::new("appleid.cdn-apple.com").unwrap());
        builder.add(Glob::new("*.apple-cloudkit.com").unwrap());
        builder.add(Glob::new("*.apple-livephotoskit.com").unwrap());
        builder.add(Glob::new("*.apzones.com").unwrap());
        builder.add(Glob::new("*.cdn-apple.com").unwrap());
        builder.add(Glob::new("*.gc.apple.com").unwrap());
        builder.add(Glob::new("*.icloud.com").unwrap());
        builder.add(Glob::new("*.icloud.com.cn").unwrap());
        builder.add(Glob::new("*.icloud.apple.com").unwrap());
        builder.add(Glob::new("*.icloud-content.com").unwrap());
        builder.add(Glob::new("*.iwork.apple.com").unwrap());
        builder.add(Glob::new("mask.icloud.com").unwrap());
        builder.add(Glob::new("mask-h2.icloud.com").unwrap());
        builder.add(Glob::new("mask-api.icloud.com").unwrap());
        builder.add(Glob::new("devimages-cdn.apple.com").unwrap());
        builder.add(Glob::new("download.developer.apple.com").unwrap());

        builder.build().unwrap()
    };
}

#[derive(Debug, Clone)]
pub struct LocalExclusionStore(Arc<RwLock<HashSet<String>>>);

impl LocalExclusionStore {
    pub fn new(exclusions: HashSet<String>) -> Self {
        Self(Arc::new(RwLock::new(exclusions)))
    }

    pub fn replace_exclusions(&mut self, exclusions: HashSet<String>) {
        *self.0.write().unwrap() = exclusions
            .into_iter()
            // Making things case insensitive
            .map(|exclusion| exclusion.to_lowercase())
            .collect();
    }

    pub fn contains(&self, element: &str) -> bool {
        // Items are stored lowercased
        let element_lowercase = element.to_lowercase();

        if DEFAULT_EXCLUSIONS.is_match(element) {
            true
        } else {
            self.0.read().unwrap().contains(&element_lowercase)
        }
    }
}
