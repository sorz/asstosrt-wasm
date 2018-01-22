use std::collections::HashMap;
use std::cmp::Ordering;
use regex::Regex;

struct DialogueFormat {
    cols: HashMap<String, usize>,
}

#[derive(PartialEq, Eq)]
struct Dialogue<'a> {
    start_cents: u32,
    end_cents: u32,
    text: &'a str,
    effect: bool,
}

impl DialogueFormat {
    fn new(line: &str) -> Result<Self, &'static str> {
        let cols = line[7..].split(',')
            .map(|c| c.trim().to_lowercase())
            .enumerate()
            .map(|(i, n)| (n, i))
            .collect::<HashMap<_, _>>();
        if !cols.contains_key("start") || !cols.contains_key("end")
                || !cols.contains_key("text") {
            return Err("Start/End/Text not found in format line");
        }
        Ok(Self { cols })
    }

    fn parse<'a>(&self, line: &'a str) -> Result<Dialogue<'a>, &'static str> {
        let cols: Vec<_> = line[9..].splitn(self.cols.len(), ',')
            .map(|c| c.trim())
            .collect();
        let start = self.cols.get("start").and_then(|i| cols.get(*i));
        let end = self.cols.get("end").and_then(|i| cols.get(*i));
        let text = self.cols.get("text").and_then(|i| cols.get(*i));
        let effect = self.cols.get("effect").and_then(|i| cols.get(*i))
            .map(|t| !t.trim().is_empty()).unwrap_or(false);
        Ok(Dialogue {
            start_cents: parse_time(start.ok_or("'Start' not found")?)
                .map_err(|_| "start time format error")?,
            end_cents: parse_time(end.ok_or("'End' not found")?)
                .map_err(|_| "end time format error")?,
            text: text.ok_or("'Text' not found")?,
            effect,
        })
    }
}

impl<'a> Dialogue<'a> {
    fn as_srt(&self) -> String {
        lazy_static! {
            static ref RE_CMD: Regex = Regex::new(r"\{.*?\}").unwrap();
        }
        let start = to_srt_time(self.start_cents);
        let end = to_srt_time(self.end_cents);
        let text = RE_CMD.replace_all(self.text, "");
        let text = text.replace(r"\n", "\r\n").replace(r"\N", "\r\n");
        format!("{} --> {}\r\n{}\r\n\r\n", start, end, text)
    }
}

impl<'a> Ord for Dialogue<'a> {
    fn cmp(&self, other: &Dialogue) -> Ordering {
        self.start_cents.cmp(&other.start_cents)
    }
}

impl<'a> PartialOrd for Dialogue<'a> {
    fn partial_cmp(&self, other: &Dialogue) -> Option<Ordering> {
        Some(self.start_cents.cmp(&other.start_cents))
    }
}

/// parse "h:mm:ss.cc" to centisec.
fn parse_time(t: &str) -> Result<u32, ()> {
    let hmsc: Vec<u32> = t.split(|c| c == ':' || c == '.')
        .filter_map(|s| s.parse().ok()).collect();
    if hmsc.len() != 4 {
        return Err(());
    }
    Ok(
        hmsc[0] * 60 * 60 * 100 +
        hmsc[1] * 60 * 100 +
        hmsc[2] * 100 +
        hmsc[3]
    )
}

/// convert centisec to "hh:mm:ss.mmm"
fn to_srt_time(t: u32) -> String {
    let h = t / 100 / 60 / 60;
    let m = t / 100 / 60 % 60;
    let s = t / 100 % 60;
    let ms = t % 100 * 10;
    format!("{:02}:{:02}:{:02},{:03}", h, m, s, ms)
}

pub fn ass_to_srt(ass: &str, no_effect: bool)
        -> Result<String, &'static str> {
    // find lines within [Events]
    let mut events = ass.lines()
        .skip_while(|l| !l.starts_with("[Events]"))
        .skip(1)
        .take_while(|l| !l.starts_with("["))
        .map(|l| l.trim());
    // find format line
    let format = events.find(|l| l.starts_with("Format:"))
        .ok_or("[Events] or Foramt line not found")?;
    let format = DialogueFormat::new(format)?;
    // parse dialogues
    let mut dialogues = events.filter(|l| l.starts_with("Dialogue:"))
        .map(|l| format.parse(l))
        .filter_map(|d| d.ok())
        .filter(|d| !no_effect || !d.effect)
        .collect::<Vec<_>>();
    // to srt
    dialogues.sort();
    Ok(dialogues.iter().map(|d| d.as_srt()).collect())
}


#[test]
fn test_ass_to_srt() {
    let ass = r#"
; 啊啊啊啊啊
[Events]
Format: Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
Dialogue: 0:02:42.42,0:02:44.05,main,b,0,0,0,,Something...
Dialogue: 0:02:40.65,0:02:41.79,main,a,0,0,0,,Hello,\nworld!~
Dialogue: 0:02:40.65,0:02:41.79,main,a,0,0,0,x,[Effect]
"#;
    let srt = "\
00:02:40,650 --> 00:02:41,790\r\n\
Hello,\r\nworld!~\r\n\r\n\
00:02:42,420 --> 00:02:44,050\r\n\
Something...\r\n\r\n";
    let result = ass_to_srt(ass, true).unwrap();
    assert_eq!(result, srt);
}

