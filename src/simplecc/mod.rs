use std::collections::HashMap;

#[cfg(test)]
mod tests;

const RAW_DICT_TS: &'static str = include_str!("ts.txt");
const RAW_DICT_ST: &'static str = include_str!("st.txt");

#[derive(Debug)]
pub struct Dict {
    root: DictNode,
}

#[derive(Debug)]
enum DictNode {
    Leaf {
        key: Box<str>,
        value: Box<str>,
    },
    Node {
        value: Option<Box<str>>,
        tails: HashMap<char, DictNode>,
    }
}

impl DictNode {
    fn new() -> Self {
        DictNode::Node {
            value: None,
            tails: HashMap::new(),
        }
    }

    fn leaf(key: &str, value: Box<str>) -> Self {
        DictNode::Leaf {
            key: key.into(),
            value,
        }
    }

    fn destruct(self) -> (Option<Box<str>>, HashMap<char, DictNode>) {
        match self {
            DictNode::Node { value, tails } => (value, tails),
            DictNode::Leaf { key, value } => {
                let mut tails = HashMap::new();
                let mut key_chars = key.chars();
                let value = if let Some(hash_key) = key_chars.next() {
                    let suffix = key_chars.as_str().into();
                    tails.insert(hash_key, DictNode::leaf(suffix, value));
                    None
                } else {
                    Some(value)
                };
                (value, tails)
            }
        }
    }

    fn add(self, key: &str, value: &str) -> Self {
        let (self_value, mut tails) = self.destruct();
        let mut key_chars = key.chars();
        if let Some(hash_key) = key_chars.next() {
            let suffix = key_chars.as_str().into();
            let node = if let Some(subnode) = tails.remove(&hash_key) {
                subnode.add(suffix, value)
            } else {
                DictNode::leaf(suffix, value.into())
            };
            tails.insert(hash_key, node);
            DictNode::Node { value: self_value, tails }
        } else {
            DictNode::leaf("", value.into())
        }
    }

    fn prefix_match<'a, 'b>(&'a self, query: &'b str)
            -> Option<(&'b str, &'a str)> {
        match self {
            &DictNode::Leaf { ref key, ref value } => {
                if query.starts_with(&**key) {
                    Some((&query[..key.len()], &value))
                } else {
                    None
                }
            },
            &DictNode::Node { ref value, ref tails } => {
                let mut query_chars = query.chars();
                let hash_key = query_chars.next();
                let suffix = query_chars.as_str();

                hash_key.and_then(|hash_key| {
                    tails.get(&hash_key)
                        .and_then(|node| node.prefix_match(suffix))
                        .map(|(prefix, value)| {
                            let n = query.len() - suffix.len() + prefix.len();
                            (&query[..n], value)
                        })
                }).or_else(||
                    value.as_ref().map(|v| ("", &**v))
                )
            }
        }
    }
}

impl Dict {
    pub fn load(raw: &str) -> Self {
        let root = raw.lines()
            .filter_map(|line| {
                let mut cols = line.splitn(2, ' ');
                Some((cols.next()?, cols.next()?))
            }).fold(DictNode::new(), |dict, (key, value)| {
                dict.add(key, value)
            });
        Dict { root }
    }

    pub fn default_t2s() -> &'static Self {
        lazy_static! {
            static ref DICT: Dict = Dict::load(RAW_DICT_TS);
        }
        &DICT
    }

    pub fn default_s2t() -> &'static Self {
        lazy_static! {
            static ref DICT: Dict = Dict::load(RAW_DICT_ST);
        }
        &DICT
    }

    pub fn replace_all(&self, mut text: &str) -> String {
        let mut output = String::with_capacity(text.len());
        while !text.is_empty() {
            match self.root.prefix_match(text) {
                Some((prefix, value)) => {
                    output.push_str(value);
                    text = &text[prefix.len()..];
                },
                None => {
                    let mut chars = text.chars();
                    output.push(chars.next().unwrap());
                    text = chars.as_str();
                }
            }
        }
        output
    }
}

