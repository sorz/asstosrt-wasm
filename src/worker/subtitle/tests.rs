use super::{Centisec, Dialogue, ass_to_srt};

const ASS_SAMPLE: &str = r#"
; 啊啊啊啊啊
[Events]
Format: Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
Dialogue: 0:02:42.42,0:02:44.05,main,b,0,0,0,,Something...
Dialogue: 0:02:40.65,0:02:41.79,main,a,0,0,0,,Hello,\nworld!~
Dialogue: 0:02:40.65,0:02:41.79,main,a,0,0,0,x,[Effect]
Dialogue: 0:03:01.00,0:03:02.00,main,a,0,0,0,,{\p1}dr{\p2}aw{\p0}
Dialogue: 0:04:01.00,0:04:02.00,main,a,0,0,0,,some{\p2}draw with{\p0}text
"#;

#[test]
fn test_cleanse_text() {
    let mut d = Dialogue {
        start: Centisec(0),
        end: Centisec(0),
        effect: false,
        text: r"some{\fad(2,5)\p1\alpha&5}few{\p2}draw{\p0}{\b0\test}text{\b1}{\p0}\Nline".into(),
    };
    d.cleanse_text();
    assert_eq!("sometext\r\nline", d.text);
}

#[test]
fn test_ass_to_srt() {
    let srt = "\
1\r\n\
00:02:40,650 --> 00:02:41,790\r\n\
Hello,\r\nworld!~\r\n\r\n\
2\r\n\
00:02:42,420 --> 00:02:44,050\r\n\
Something...\r\n\r\n\
3\r\n\
00:04:01,000 --> 00:04:02,000\r\n\
sometext\r\n\r\n";
    let conv = |s| Some(s);
    let result = ass_to_srt(ASS_SAMPLE, true, Some(conv), 0.0).unwrap();
    assert_eq!(result, srt);
}

#[test]
fn test_ass_line_ending() {
    let crlf = ASS_SAMPLE.replace('\n', "\r\n");
    let lf = ASS_SAMPLE.replace('\n', "\r");
    let conv = |s| Some(s);
    assert!(ass_to_srt(&crlf, true, Some(conv), 0.0).is_ok());
    assert!(ass_to_srt(&lf, true, Some(conv), 0.0).is_ok());
}
