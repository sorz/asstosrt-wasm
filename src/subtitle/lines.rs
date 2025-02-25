/// std::str::Lines that also split at single '\r'
pub(crate) struct UniversalLines<'a> {
    text: &'a str,
    pos: usize,
}

impl<'a> UniversalLines<'a> {
    pub(crate) fn new(text: &'a str) -> Self {
        Self { text, pos: 0 }
    }
}

impl<'a> Iterator for UniversalLines<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        let start = self.pos;
        let end = self.text[self.pos..]
            .find(['\r', '\n'])
            .map(|n| start + n)
            .unwrap_or(self.text.len());
        self.pos = if self.text[end..].starts_with("\r\n") {
            end + 2
        } else {
            // '\r' or '\n'
            end + 1
        }
        .clamp(start, self.text.len());
        if start == self.text.len() {
            None
        } else {
            Some(&self.text[start..end])
        }
    }
}

#[test]
fn test_universal_lines() {
    let mut lines = UniversalLines::new("a\r\nb\nc\r\r\n\n\r\n\r\n");
    assert_eq!(lines.next(), Some("a")); // a\r\n
    assert_eq!(lines.next(), Some("b")); // b\n
    assert_eq!(lines.next(), Some("c")); // c\r
    assert_eq!(lines.next(), Some("")); // \r
    assert_eq!(lines.next(), Some("")); // \n
    assert_eq!(lines.next(), Some("")); // \n
    assert_eq!(lines.next(), Some("")); // \r\n
    assert_eq!(lines.next(), None);
    assert_eq!(lines.next(), None);

    let mut lines = UniversalLines::new("single line");
    assert_eq!(lines.next(), Some("single line"));
    assert_eq!(lines.next(), None);

    let mut lines = UniversalLines::new("");
    assert_eq!(lines.next(), None);
}
