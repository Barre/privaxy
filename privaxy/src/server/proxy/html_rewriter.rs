use crate::{blocker::AdblockRequester, statistics::Statistics};
use crossbeam_channel::Receiver;
use hyper::body::Bytes;
use lol_html::{element, HtmlRewriter, Settings};
use regex::Regex;
use std::collections::HashSet;
use tokio::sync;

type InternalBodyChannel = (
    sync::mpsc::UnboundedSender<(Bytes, Option<AdblockProperties>)>,
    sync::mpsc::UnboundedReceiver<(Bytes, Option<AdblockProperties>)>,
);

struct AdblockProperties {
    url: String,
    ids: HashSet<String>,
    classes: HashSet<String>,
}

pub struct Rewriter {
    url: String,
    adblock_requester: AdblockRequester,
    receiver: Receiver<Bytes>,
    body_sender: hyper::body::Sender,
    statistics: Statistics,
    internal_body_channel: InternalBodyChannel,
}

impl Rewriter {
    pub(crate) fn new(
        url: String,
        adblock_requester: AdblockRequester,
        receiver: Receiver<Bytes>,
        body_sender: hyper::body::Sender,
        statistics: Statistics,
    ) -> Self {
        Self {
            url,
            body_sender,
            statistics,
            adblock_requester,
            receiver,
            internal_body_channel: sync::mpsc::unbounded_channel(),
        }
    }

    pub(crate) fn rewrite(self) {
        let (internal_body_sender, internal_body_receiver) = self.internal_body_channel;
        let body_sender = self.body_sender;
        let adblock_requester = self.adblock_requester.clone();
        let statistics = self.statistics.clone();

        let mut classes = HashSet::new();
        let mut ids = HashSet::new();

        tokio::spawn(Self::write_body(
            internal_body_receiver,
            body_sender,
            adblock_requester,
            statistics,
        ));

        let mut rewriter = HtmlRewriter::new(
            Settings {
                element_content_handlers: vec![
                    element!("*", |element| {
                        let id = element.get_attribute("id");

                        if let Some(id) = id {
                            ids.insert(id);
                        }

                        Ok(())
                    }),
                    element!("*", |element| {
                        let class = element.get_attribute("class");

                        if let Some(class) = class {
                            let re = Regex::new(r"\s+").unwrap();
                            let classes_without_duplicate_spaces = re.replace_all(&class, " ");

                            let class = classes_without_duplicate_spaces
                                .split(' ')
                                .map(|s| s.to_string())
                                .collect::<HashSet<_>>();

                            classes.extend(class);
                        }

                        Ok(())
                    }),
                ],
                ..Settings::default()
            },
            |c: &[u8]| {
                let _result = internal_body_sender.send((Bytes::copy_from_slice(c), None));
            },
        );

        for message in self.receiver {
            rewriter.write(&message).unwrap();
        }
        rewriter.end().unwrap();

        let _result = internal_body_sender.send((
            Bytes::new(),
            Some(AdblockProperties {
                ids,
                classes,
                url: self.url,
            }),
        ));
    }

    async fn write_body(
        mut receiver: sync::mpsc::UnboundedReceiver<(Bytes, Option<AdblockProperties>)>,
        mut body_sender: hyper::body::Sender,
        adblock_requester: AdblockRequester,
        statistics: Statistics,
    ) {
        while let Some((bytes, adblock_properties)) = receiver.recv().await {
            if let Err(_err) = body_sender.send_data(bytes).await {
                break;
            }
            if let Some(adblock_properties) = adblock_properties {
                let mut response_has_been_modified = false;

                let blocker_result = adblock_requester
                    .get_cosmetic_response(
                        adblock_properties.url,
                        Vec::from_iter(adblock_properties.ids.into_iter()),
                        Vec::from_iter(adblock_properties.classes.into_iter()),
                    )
                    .await;

                let mut to_append_to_response = format!(
                    r#"
<!-- privaxy proxy -->
<style>{hidden_selectors} {display_none}
{style_selectors}
</style>
<!-- privaxy proxy -->"#,
                    display_none = {
                        if blocker_result.hidden_selectors.is_empty() {
                            ""
                        } else {
                            response_has_been_modified = true;

                            "{ display: none !important;} "
                        }
                    },
                    hidden_selectors = blocker_result.hidden_selectors.join(","),
                    style_selectors = {
                        let style_selectors = blocker_result.style_selectors;

                        if !style_selectors.is_empty() {
                            response_has_been_modified = true
                        }

                        style_selectors
                            .into_iter()
                            .map(|(selector, content)| {
                                format!(
                                    "{selector} {{ {content} }}",
                                    selector = selector,
                                    content = content.join(";")
                                )
                            })
                            .collect::<String>()
                    }
                );

                if let Some(injected_script) = blocker_result.injected_script {
                    response_has_been_modified = true;

                    to_append_to_response.push_str(&format!(
                        r#"
<!-- Privaxy proxy -->
<script type="application/javascript">{}</script>
<!-- privaxy proxy -->
"#,
                        injected_script
                    ))
                }

                if response_has_been_modified {
                    statistics.increment_modified_responses();
                }

                let bytes = Bytes::copy_from_slice(&to_append_to_response.into_bytes());

                if let Err(_err) = body_sender.send_data(bytes).await {
                    break;
                }
            }
        }
    }
}
