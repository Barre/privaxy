///! Implementation of following utils are derived from https://github.com/brave/adblock-rust/blob/e508d3ecc0c6e5316d860f7d0829c6b2d0b3064f/src/resources/resource_assembler.rs
use adblock::resources::{MimeType, Resource, ResourceType};
use once_cell::sync::Lazy;
use regex::Regex;

static TOP_COMMENT_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r#"^/\*[\S\s]+?\n\*/\s*"#).unwrap());
static NON_EMPTY_LINE_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r#"\S"#).unwrap());
static MAP_END_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r#"^\s*\]\s*\)"#).unwrap());

static TRAILING_COMMA_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r#",([\],\}])"#).unwrap());
static UNQUOTED_FIELD_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r#"([\{,])([a-zA-Z][a-zA-Z0-9_]*):"#).unwrap());

const REDIRECTABLE_RESOURCES_DECLARATION: &str = "const redirectableResources = new Map([";

/// Maps the name of the resource to its properties in a 2-element tuple.
type JsResourceEntry = (String, JsResourceProperties);

//  ]);
/// Represents a single entry of the `redirectableResources` map from uBlock Origin's
/// `redirect-engine.js`.
///
/// - `name` is the name of a resource, corresponding to its path in the `web_accessible_resources`
/// directory
///
/// - `alias` is a list of optional additional names that can be used to reference the resource
///
/// - `data` is either `"text"` or `"blob"`, but is currently unused in `adblock-rust`. Within
/// uBlock Origin, it's used to prevent text files from being encoded in base64 in a data URL.
pub struct ResourceProperties {
    pub name: String,
    pub alias: Vec<String>,
    pub data: Option<String>,
}

/// Directly deserializable representation of a resource's properties from `redirect-engine.js`.
#[derive(serde::Deserialize)]
struct JsResourceProperties {
    #[serde(default)]
    alias: Option<ResourceAliasField>,
    #[serde(default)]
    data: Option<String>,
    #[serde(default)]
    params: Option<Vec<String>>,
}

/// The deserializable represenation of the `alias` field of a resource's properties, which can
/// either be a single string or a list of strings.
#[derive(serde::Deserialize)]
#[serde(untagged)]
enum ResourceAliasField {
    SingleString(String),
    ListOfStrings(Vec<String>),
}

impl ResourceAliasField {
    fn to_vec(self) -> Vec<String> {
        match self {
            Self::SingleString(s) => vec![s],
            Self::ListOfStrings(l) => l,
        }
    }
}

pub fn read_template_resources(scriptlets_data: &str) -> Vec<Resource> {
    let mut resources = Vec::new();

    let uncommented = TOP_COMMENT_RE.replace_all(&scriptlets_data, "");
    let mut name: Option<&str> = None;
    let mut details = std::collections::HashMap::<_, Vec<_>>::new();
    let mut script = String::new();

    for line in uncommented.lines() {
        if line.starts_with('#') || line.starts_with("// ") || line == "//" {
            continue;
        }

        if name.is_none() {
            if let Some(stripped) = line.strip_prefix("/// ") {
                name = Some(stripped.trim());
            }
            continue;
        }

        if let Some(stripped) = line.strip_prefix("/// ") {
            let mut line = stripped.split_whitespace();
            let prop = line.next().expect("Detail line has property name");
            let value = line.next().expect("Detail line has property value");
            details
                .entry(prop)
                .and_modify(|v| v.push(value))
                .or_insert_with(|| vec![value]);
            continue;
        }

        if NON_EMPTY_LINE_RE.is_match(line) {
            script += line.trim();
            script.push('\n');
            continue;
        }

        let kind = if script.contains("{{1}}") {
            ResourceType::Template
        } else {
            ResourceType::Mime(MimeType::ApplicationJavascript)
        };

        resources.push(Resource {
            name: name.expect("Resource name must be specified").to_owned(),
            aliases: details
                .get("alias")
                .map(|aliases| aliases.iter().map(|alias| alias.to_string()).collect())
                .unwrap_or_default(),
            kind,
            content: base64::encode(&script),
        });

        name = None;
        details.clear();
        script.clear();
    }

    resources
}

/// Reads data from a a file in the format of uBlock Origin's `redirect-engine.js` file to
/// determine the files in the `web_accessible_resources` directory, as well as any of their
/// aliases.
///
/// This is read from the `redirectableResources` map.
pub fn read_redirectable_resource_mapping(mapfile_data: &str) -> Vec<ResourceProperties> {
    // This isn't bulletproof, but it should handle the historical versions of the mapping
    // correctly, and having a strict JSON parser should catch any unexpected format changes. Plus,
    // it prevents dependending on a full JS engine.

    // Extract just the map. It's between REDIRECTABLE_RESOURCES_DECLARATION and MAP_END_RE.
    let mut map: String = mapfile_data
        .lines()
        .skip_while(|line| *line != REDIRECTABLE_RESOURCES_DECLARATION)
        .take_while(|line| !MAP_END_RE.is_match(line))
        // Strip any trailing comments from each line.
        .map(|line| {
            if let Some(i) = line.find("//") {
                &line[..i]
            } else {
                line
            }
        })
        // Remove all newlines from the entire string.
        .fold(String::new(), |s, line| s + line);

    // Add back the final square brace that was omitted above as part of MAP_END_RE.
    map.push(']');

    // Trim out the beginning `const redirectableResources = new Map(`.
    // Also, replace all single quote characters with double quotes.
    assert!(map.starts_with(REDIRECTABLE_RESOURCES_DECLARATION));
    map = map[REDIRECTABLE_RESOURCES_DECLARATION.len() - 1..].replace('\'', "\"");

    // Remove all whitespace from the entire string.
    map.retain(|c| !c.is_whitespace());

    // Replace all matches for `,]` or `,}` with `]` or `}`, respectively.
    map = TRAILING_COMMA_RE
        .replace_all(&map, |caps: &regex::Captures| caps[1].to_string())
        .to_string();

    // Replace all property keys directly preceded by a `{` or a `,` and followed by a `:` with
    // double-quoted versions.
    map = UNQUOTED_FIELD_RE
        .replace_all(&map, |caps: &regex::Captures| {
            format!("{}\"{}\":", &caps[1], &caps[2])
        })
        .to_string();

    // It *should* be valid JSON now, so parse it with serde_json.
    let parsed: Vec<JsResourceEntry> = serde_json::from_str(&map).unwrap();

    parsed
        .into_iter()
        .filter_map(|(name, props)| {
            // Ignore resources with params for now, since there's no support for them currently.
            if props.params.is_some() {
                None
            } else {
                Some(ResourceProperties {
                    name,
                    alias: props.alias.map(|a| a.to_vec()).unwrap_or_default(),
                    data: props.data,
                })
            }
        })
        .collect()
}

/// Reads byte data from an arbitrary resource file, and assembles a `Resource` from it with the
/// provided `resource_info`.
pub fn build_resource_from_file_contents(
    resource_contents: &[u8],
    resource_info: &ResourceProperties,
) -> Resource {
    let name = resource_info.name.to_owned();
    let aliases = resource_info
        .alias
        .iter()
        .map(|alias| alias.to_string())
        .collect();
    let mimetype = MimeType::from_extension(&resource_info.name[..]);
    let content = match mimetype {
        MimeType::ApplicationJavascript | MimeType::TextHtml | MimeType::TextPlain => {
            let utf8string = std::str::from_utf8(resource_contents).unwrap();
            base64::encode(&utf8string.replace('\r', ""))
        }
        _ => base64::encode(&resource_contents),
    };

    Resource {
        name,
        aliases,
        kind: ResourceType::Mime(mimetype),
        content,
    }
}
