use std::collections::HashMap;

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


#[test]
fn test_prefix_match() {
    let dict = Dict::load("
A a'
B b'
C c'
ABC abc'
ABCD abcd'
DDD ddd'
BB bb'");
    assert_eq!(Some(("A", "a'")), dict.root.prefix_match("A"));
    assert_eq!(Some(("B", "b'")), dict.root.prefix_match("BXX"));
    assert_eq!(Some(("ABC", "abc'")), dict.root.prefix_match("ABCX"));
    assert_eq!(Some(("ABCD", "abcd'")), dict.root.prefix_match("ABCDEFG"));
    assert_eq!(None, dict.root.prefix_match("X"));
    assert_eq!(None, dict.root.prefix_match("DD"));
}

#[test]
fn test_dict_simple() {
    let dict = Dict::load("
A a
B b
ABC xxx
'");
    assert_eq!("a", dict.replace_all("A"));
    assert_eq!("ab", dict.replace_all("AB"));
    assert_eq!("xxx", dict.replace_all("ABC"));
    assert_eq!("abxxxa", dict.replace_all("ABABCA"));
    assert_eq!("aXbXab", dict.replace_all("AXBXAB"));
}

#[test]
fn test_dict_ts() {
    let tc = "曾經有一份真誠的愛情放在我面前，我沒有珍惜，等我失去的時候我才\
    後悔莫及。人事間最痛苦的事莫過於此。如果上天能夠給我一個再來一次得機會，\
    我會對那個女孩子說三個字，我愛你。如果非要在這份愛上加個期限，我希望是，\
    一萬年。";
    let sc = "曾经有一份真诚的爱情放在我面前，我没有珍惜，等我失去的时候我才\
    后悔莫及。人事间最痛苦的事莫过于此。如果上天能够给我一个再来一次得机会，\
    我会对那个女孩子说三个字，我爱你。如果非要在这份爱上加个期限，我希望是，\
    一万年。";
    assert_eq!(sc, Dict::default_t2s().replace_all(tc));
}

#[test]
fn test_dict_st() {
    let sc = "夸夸其谈 夸父逐日
    我干什么不干你事。
    太后的头发很干燥。
    燕燕于飞，差池其羽。之子于归，远送于野。
    请成相，世之殃，愚暗愚暗堕贤良。人主无贤，如瞽无相何伥伥！请布基，慎圣人\
    ，愚而自专事不治。主忌苟胜，群臣莫谏必逢灾。
    曾经有一份真诚的爱情放在我面前，我没有珍惜，等我失去的时候我才后悔莫及。\
    人事间最痛苦的事莫过于此。如果上天能够给我一个再来一次得机会，我会对那个\
    女孩子说三个字，我爱你。如果非要在这份爱上加个期限，我希望是，一万年。
    新的理论被发现了。
    鲶鱼和鲇鱼是一种生物。
    金胄不是金色的甲胄。";
    let tc = "誇誇其談 夸父逐日
    我幹什麼不干你事。
    太后的頭髮很乾燥。
    燕燕于飛，差池其羽。之子于歸，遠送於野。
    請成相，世之殃，愚闇愚闇墮賢良。人主無賢，如瞽無相何倀倀！請布基，慎聖人\
    ，愚而自專事不治。主忌苟勝，羣臣莫諫必逢災。
    曾經有一份真誠的愛情放在我面前，我沒有珍惜，等我失去的時候我才後悔莫及。\
    人事間最痛苦的事莫過於此。如果上天能夠給我一個再來一次得機會，我會對那個\
    女孩子說三個字，我愛你。如果非要在這份愛上加個期限，我希望是，一萬年。
    新的理論被發現了。
    鮎魚和鮎魚是一種生物。
    金胄不是金色的甲冑。";
    assert_eq!(tc, Dict::default_s2t().replace_all(sc));
}

