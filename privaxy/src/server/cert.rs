use http::uri::Authority;
use openssl::{
    asn1::Asn1Time,
    bn::{BigNum, MsbOption},
    hash::MessageDigest,
    pkey::{PKey, PKeyRef, Private},
    rsa::Rsa,
    x509::{
        extension::{
            AuthorityKeyIdentifier, BasicConstraints, KeyUsage, SubjectAlternativeName,
            SubjectKeyIdentifier,
        },
        X509NameBuilder, X509Ref, X509Req, X509ReqBuilder, X509,
    },
};
use rustls::{Certificate, PrivateKey, ServerConfig};
use std::{str::FromStr, sync::Arc};
use tokio::sync::Mutex;
use uluru::LRUCache;

const MAX_CACHED_CERTIFICATES: usize = 1_000;

#[derive(Clone)]
pub struct SignedWithCaCert {
    authority: Authority,
    pub server_configuration: ServerConfig,
}

impl SignedWithCaCert {
    fn new(
        authority: Authority,
        private_key: PKey<Private>,
        ca_certificate: X509,
        ca_private_key: PKey<Private>,
    ) -> Self {
        let x509 =
            Self::build_ca_signed_cert(&ca_certificate, &ca_private_key, &authority, &private_key);

        let certs = vec![
            Certificate(x509.to_der().unwrap()),
            Certificate(ca_certificate.to_der().unwrap()),
        ];

        let server_configuration = ServerConfig::builder()
            .with_safe_default_cipher_suites()
            .with_safe_default_kx_groups()
            .with_safe_default_protocol_versions()
            .unwrap()
            .with_no_client_auth()
            .with_single_cert(certs, PrivateKey(private_key.private_key_to_der().unwrap()))
            .unwrap();

        Self {
            authority,
            server_configuration,
        }
    }

    fn build_certificate_request(key_pair: &PKey<Private>, authority: &Authority) -> X509Req {
        let mut request_builder = X509ReqBuilder::new().unwrap();
        request_builder.set_pubkey(key_pair).unwrap();

        let mut x509_name = X509NameBuilder::new().unwrap();

        // Only 64 characters are allowed in the CN field.
        // (ub-common-name INTEGER ::= 64), browsers are not using CN anymore but uses SANs instead.
        // Let's use a shorter entry.
        // RFC 3280.
        let authority_host = authority.host();
        let common_name = if authority_host.len() > 64 {
            "privaxy_cn_too_long.local"
        } else {
            authority_host
        };

        x509_name.append_entry_by_text("CN", common_name).unwrap();
        let x509_name = x509_name.build();
        request_builder.set_subject_name(&x509_name).unwrap();

        request_builder
            .sign(key_pair, MessageDigest::sha256())
            .unwrap();

        request_builder.build()
    }

    fn build_ca_signed_cert(
        ca_cert: &X509Ref,
        ca_key_pair: &PKeyRef<Private>,
        authority: &Authority,
        private_key: &PKey<Private>,
    ) -> X509 {
        let req = Self::build_certificate_request(private_key, authority);

        let mut cert_builder = X509::builder().unwrap();
        cert_builder.set_version(2).unwrap();

        let serial_number = {
            let mut serial = BigNum::new().unwrap();
            serial.rand(159, MsbOption::MAYBE_ZERO, false).unwrap();
            serial.to_asn1_integer().unwrap()
        };

        cert_builder.set_serial_number(&serial_number).unwrap();
        cert_builder.set_subject_name(req.subject_name()).unwrap();
        cert_builder
            .set_issuer_name(ca_cert.subject_name())
            .unwrap();
        cert_builder.set_pubkey(private_key).unwrap();

        let not_before = Asn1Time::days_from_now(0).unwrap();
        cert_builder.set_not_before(&not_before).unwrap();

        let not_after = Asn1Time::days_from_now(365).unwrap();
        cert_builder.set_not_after(&not_after).unwrap();

        cert_builder
            .append_extension(BasicConstraints::new().build().unwrap())
            .unwrap();

        cert_builder
            .append_extension(
                KeyUsage::new()
                    .critical()
                    .non_repudiation()
                    .digital_signature()
                    .key_encipherment()
                    .build()
                    .unwrap(),
            )
            .unwrap();

        let subject_alternative_name = match std::net::IpAddr::from_str(authority.host()) {
            // If we are able to parse the authority as an ip address, let's build an "IP" field instead
            // of a "DNS" one.
            Ok(_ip_addr) => {
                let mut san = SubjectAlternativeName::new();
                san.ip(authority.host());

                san
            }
            Err(_err) => {
                let mut san = SubjectAlternativeName::new();
                san.dns(authority.host());
                san
            }
        }
        .build(&cert_builder.x509v3_context(Some(ca_cert), None))
        .unwrap();

        cert_builder
            .append_extension(subject_alternative_name)
            .unwrap();

        let subject_key_identifier = SubjectKeyIdentifier::new()
            .build(&cert_builder.x509v3_context(Some(ca_cert), None))
            .unwrap();
        cert_builder
            .append_extension(subject_key_identifier)
            .unwrap();

        let auth_key_identifier = AuthorityKeyIdentifier::new()
            .keyid(false)
            .issuer(false)
            .build(&cert_builder.x509v3_context(Some(ca_cert), None))
            .unwrap();
        cert_builder.append_extension(auth_key_identifier).unwrap();

        cert_builder
            .sign(ca_key_pair, MessageDigest::sha256())
            .unwrap();

        cert_builder.build()
    }
}

#[derive(Clone)]
pub struct CertCache {
    cache: Arc<Mutex<LRUCache<SignedWithCaCert, MAX_CACHED_CERTIFICATES>>>,
    // We use a single RSA key for all certificates.
    private_key: PKey<Private>,
    ca_certificate: X509,
    ca_private_key: PKey<Private>,
}

impl CertCache {
    pub fn new(ca_certificate: X509, ca_private_key: PKey<Private>) -> Self {
        Self {
            cache: Arc::new(Mutex::new(LRUCache::default())),
            private_key: {
                let rsa = Rsa::generate(2048).unwrap();
                PKey::from_rsa(rsa).unwrap()
            },
            ca_certificate,
            ca_private_key,
        }
    }

    async fn insert(&self, certificate: SignedWithCaCert) {
        let mut cache = self.cache.lock().await;
        cache.insert(certificate);
    }

    pub async fn get(&self, authority: Authority) -> SignedWithCaCert {
        let mut cache = self.cache.lock().await;

        match cache.find(|cert| cert.authority == authority) {
            Some(certificate) => certificate.clone(),
            None => {
                // We release the previously acquired lock early as `insert`, which we will call just
                // afterwards also waits to acquire a lock.
                std::mem::drop(cache);

                let private_key = self.private_key.clone();

                let ca_certificate = self.ca_certificate.clone();
                let ca_private_key = self.ca_private_key.clone();

                // This operation is somewhat CPU intensive and on some lower powered machines,
                // not running it inside of a thread pool may cause it to block the executor for too long.
                let certificate = tokio::task::spawn_blocking(move || {
                    SignedWithCaCert::new(authority, private_key, ca_certificate, ca_private_key)
                })
                .await
                .unwrap();

                self.insert(certificate.clone()).await;
                certificate
            }
        }
    }
}
