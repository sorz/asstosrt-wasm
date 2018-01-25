use super::{Dialogue, Centisec, ass_to_srt};

#[test]
fn test_cleanse_text() {
    let mut d = Dialogue {
        start: Centisec(0), end: Centisec(0), effect: false,
        text: r"some{\p1}few{\p2}draw{\p0}{\b0}text{\b1}{\p0}\Nline".into(),
    };
    d.cleanse_text();
    assert_eq!("sometext\r\nline", d.text);
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
Dialogue: 0:03:01.00,0:03:02.00,main,a,0,0,0,,{\p1}dr{\p2}aw{\p0}
Dialogue: 0:04:01.00,0:04:02.00,main,a,0,0,0,,some{\p2}draw with{\p0}text
"#;
    let srt = "\
00:02:40,650 --> 00:02:41,790\r\n\
Hello,\r\nworld!~\r\n\r\n\
00:02:42,420 --> 00:02:44,050\r\n\
Something...\r\n\r\n\
00:04:01,000 --> 00:04:02,000\r\n\
sometext\r\n\r\n";
    let conv = |s| Some(s);
    let result = ass_to_srt(ass, true, Some(conv)).unwrap();
    assert_eq!(result, srt);
}

