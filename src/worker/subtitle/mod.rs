use lazy_static::lazy_static;
use lines::UniversalLines;
use regex_lite::Regex;
use serde::{Deserialize, Serialize};
use std::{borrow::Cow, cmp::Ordering, collections::HashMap, fmt, str::FromStr};
use strum::AsRefStr;
use thiserror::Error;

mod lines;
#[cfg(test)]
mod tests;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, AsRefStr)]
pub enum Field {
    #[strum(serialize = "start")]
    Start,
    #[strum(serialize = "end")]
    End,
    #[strum(serialize = "text")]
    Text,
    #[strum(serialize = "effect")]
    Effect,
}

#[derive(Error, Debug, Clone, Serialize, Deserialize)]
pub enum FormatError {
    #[error("`[Events] => Format` line not found")]
    NoFormatLine,
    #[error("field `{0:?}` not found in `Format` line")]
    NoFormatLineField(Field),
    #[error("field `{0:?}` not found in dialogue")]
    NoField(Field),
    #[error("failed to parse time `{0}`")]
    Time(String),
}

struct DialogueFormat {
    cols: HashMap<String, usize>,
}

#[derive(PartialEq, Eq, PartialOrd, Ord)]
struct Centisec(u32);

#[derive(PartialEq, Eq)]
struct Dialogue<'a> {
    start: Centisec,
    end: Centisec,
    text: Cow<'a, str>,
    effect: bool,
}

impl DialogueFormat {
    fn new(line: &str) -> Result<Self, FormatError> {
        let cols = line[7..]
            .split(',')
            .map(|c| c.trim().to_lowercase())
            .enumerate()
            .map(|(i, n)| (n, i))
            .collect::<HashMap<_, _>>();
        for field in [Field::Start, Field::End, Field::Text] {
            if !cols.contains_key(field.as_ref()) {
                return Err(FormatError::NoFormatLineField(field));
            }
        }
        Ok(Self { cols })
    }

    fn parse<'a>(&self, line: &'a str) -> Result<Dialogue<'a>, FormatError> {
        let cols: Vec<_> = line[9..]
            .splitn(self.cols.len(), ',')
            .map(|c| c.trim())
            .collect();
        let opt_field = |field: Field| self.cols.get(field.as_ref()).and_then(|i| cols.get(*i));
        let req_field = |field: Field| opt_field(field).ok_or(FormatError::NoField(field));

        let start = req_field(Field::Start)?.parse()?;
        let end = req_field(Field::End)?.parse()?;
        let text = req_field(Field::Text)?;
        let effect = opt_field(Field::Effect)
            .map(|t| !t.trim().is_empty())
            .unwrap_or(false);
        Ok(Dialogue {
            start,
            end,
            effect,
            text: Cow::from(*text),
        })
    }
}

impl Dialogue<'_> {
    fn cleanse_text(&mut self) {
        lazy_static! {
            static ref RE_CMD: Regex = Regex::new(
                // remove:
                // {\pX}...{\p0} or {\pX}... (draw cmd); and
                // {...} (other cmds}
                r"\{[^\}]*\\p[1-9][^\}]*\}.*?(\{[^\}]*\\p0[^\}]*\}|$)|\{.*?\}"
            ).unwrap();
            static ref RE_LINE: Regex = Regex::new(
                r"\\[Nn]"
            ).unwrap();
        }
        self.text = {
            let text = RE_CMD.replace_all(&self.text, "");
            let text = RE_LINE.replace_all(&text, "\r\n");
            text.into_owned()
        }
        .into();
    }

    fn as_srt(&self, id: usize) -> String {
        format!(
            "{}\r\n{} --> {}\r\n{}\r\n\r\n",
            id, self.start, self.end, self.text
        )
    }
}

impl Ord for Dialogue<'_> {
    fn cmp(&self, other: &Dialogue) -> Ordering {
        self.start.cmp(&other.start)
    }
}

impl PartialOrd for Dialogue<'_> {
    fn partial_cmp(&self, other: &Dialogue) -> Option<Ordering> {
        Some(self.start.cmp(&other.start))
    }
}

/// parse "h:mm:ss.cc" to centisec.
impl FromStr for Centisec {
    type Err = FormatError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let hmsc: Vec<u32> = s.split([':', '.']).filter_map(|s| s.parse().ok()).collect();
        if hmsc.len() == 4 {
            Ok(Centisec(
                hmsc[0] * 60 * 60 * 100 + hmsc[1] * 60 * 100 + hmsc[2] * 100 + hmsc[3],
            ))
        } else {
            Err(FormatError::Time(s.to_string()))
        }
    }
}

/// convert centisecs to "hh:mm:ss.mmm"
impl fmt::Display for Centisec {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        let t = self.0;
        let h = t / 100 / 60 / 60;
        let m = t / 100 / 60 % 60;
        let s = t / 100 % 60;
        let ms = t % 100 * 10;
        write!(f, "{:02}:{:02}:{:02},{:03}", h, m, s, ms)
    }
}

impl Centisec {
    fn add_secs(&mut self, secs: f32) {
        let secs = self.0 as f32 + (secs * 100.0);
        self.0 = if secs <= 0.0 { 0 } else { secs.round() as u32 }
    }
}

pub fn ass_to_srt<'a: 'b, 'b, F>(
    ass: &'a str,
    no_effect: bool,
    mut mapper: Option<F>,
    offset_secs: f32,
) -> Result<String, FormatError>
where
    F: FnMut(Cow<'b, str>) -> Cow<'b, str>,
{
    // find lines within [Events]
    let mut events = UniversalLines::new(ass)
        .skip_while(|l| !l.starts_with("[Events]"))
        .skip(1)
        .take_while(|l| !l.starts_with("["))
        .map(|l| l.trim());
    // find format line
    let format = events
        .find(|l| l.starts_with("Format:"))
        .ok_or(FormatError::NoFormatLine)?;
    let format = DialogueFormat::new(format)?;
    // parse dialogues
    let mut dialogues = events
        .filter(|l| l.starts_with("Dialogue:"))
        .map(|l| format.parse(l))
        .filter_map(|d| d.ok())
        .filter(|d| !no_effect || !d.effect)
        .collect::<Vec<_>>();
    // to srt
    dialogues.sort();
    let mut id = 0;
    Ok(dialogues
        .into_iter()
        .filter_map(|mut d| {
            d.cleanse_text();
            if d.text.is_empty() {
                return None;
            }
            d.start.add_secs(offset_secs);
            d.end.add_secs(offset_secs);
            if let Some(ref mut f) = mapper {
                d.text = f(d.text);
            }
            Some(d)
        })
        .filter(|d| d.end.0 > d.start.0)
        .map(|d| {
            id += 1;
            d.as_srt(id)
        })
        .collect())
}
