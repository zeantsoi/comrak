use ctype::{ispunct, isspace};
use entity;
use parser::AutolinkType;
use tendril::Tendril;
use tendril::fmt::UTF8;

pub fn unescape(v: &mut String) {
    let mut r = 0;
    let mut sz = v.len();

    while r < sz {
        if v.as_bytes()[r] == b'\\' && r + 1 < sz && ispunct(v.as_bytes()[r + 1]) {
            v.remove(r);
            sz -= 1;
        }
        if r >= sz {
            break;
        }
        r += 1;
    }
}

pub fn clean_autolink(mut url: Tendril<UTF8>, kind: AutolinkType) -> Tendril<UTF8> {
    trim(&mut url);

    if url.is_empty() {
        return url;
    }

    let mut buf = String::with_capacity(url.len());
    if kind == AutolinkType::Email {
        buf += "mailto:";
    }

    buf += &entity::unescape_html(&url);
    buf.into()
}

pub fn normalize_whitespace(v: Tendril<UTF8>) -> Tendril<UTF8> {
    let mut last_char_was_space = false;
    let mut r = String::with_capacity(v.len());

    for c in v.chars() {
        if (c as u32) < 0x80 && isspace(c as u8) {
            if !last_char_was_space {
                r.push(' ');
                last_char_was_space = true;
            }
        } else {
            r.push(c);
            last_char_was_space = false;
        }
    }

    r.into()
}

pub fn remove_trailing_blank_lines(line: &mut Tendril<UTF8>) {
    let mut i = line.len32() - 1;
    loop {
        let c = line.as_bytes()[i as usize];

        if c != b' ' && c != b'\t' && !is_line_end_char(c) {
            break;
        }

        if i == 0 {
            line.clear();
            return;
        }

        i -= 1;
    }

    for i in i..line.len32() {
        let c = line.as_bytes()[i as usize];

        if !is_line_end_char(c) {
            continue;
        }

        if i != line.len32() {
            let len = line.len32();
            line.pop_back(len - i);
        }
        break;
    }
}

pub fn is_line_end_char(ch: u8) -> bool {
    match ch {
        10 | 13 => true,
        _ => false,
    }
}

pub fn is_space_or_tab(ch: u8) -> bool {
    match ch {
        9 | 32 => true,
        _ => false,
    }
}

pub fn chop_trailing_hashtags(line: &mut Tendril<UTF8>) {
    rtrim(line);

    let orig_n = line.len() - 1;
    let mut n = orig_n;

    while line.as_bytes()[n] == b'#' {
        if n == 0 {
            return;
        }
        n -= 1;
    }

    if n != orig_n && is_space_or_tab(line.as_bytes()[n]) {
        if line.len32() != n as u32 {
            let len = line.len32();
            line.pop_back(len - n as u32);
        }
        rtrim(line);
    }
}

pub fn rtrim(line: &mut Tendril<UTF8>) {
    let mut len = line.len();
    while len > 0 && isspace(line.as_bytes()[len - 1]) {
        line.pop_back(1);
        len -= 1;
    }
}

pub fn ltrim(line: &mut Tendril<UTF8>) {
    let mut len = line.len();
    while len > 0 && isspace(line.as_bytes()[0]) {
        line.pop_front(0);
        len -= 1;
    }
}

pub fn trim(line: &mut Tendril<UTF8>) {
    ltrim(line);
    rtrim(line);
}

pub fn trim_slice(mut i: &str) -> &str {
    let mut len = i.len();
    while len > 0 && isspace(i.as_bytes()[0]) {
        i = &i[1..];
        len -= 1;
    }
    while len > 0 && isspace(i.as_bytes()[len - 1]) {
        i = &i[..len - 1];
        len -= 1;
    }
    i
}

pub fn clean_url(url: &str) -> String {
    let url = trim_slice(url);

    let url_len = url.len();
    if url_len == 0 {
        return String::new();
    }

    let mut b = if url.as_bytes()[0] == b'<' && url.as_bytes()[url_len - 1] == b'>' {
        entity::unescape_html(&url[1..url_len - 1])
    } else {
        entity::unescape_html(url)
    };

    unescape(&mut b);
    b
}

pub fn clean_title(title: &str) -> String {
    let title_len = title.len();
    if title_len == 0 {
        return String::new();
    }

    let first = title.as_bytes()[0];
    let last = title.as_bytes()[title_len - 1];

    let mut b = if (first == b'\'' && last == b'\'') || (first == b'(' && last == b')') ||
                   (first == b'"' && last == b'"') {
        entity::unescape_html(&title[1..title_len - 1])
    } else {
        entity::unescape_html(title)
    };

    unescape(&mut b);
    b
}

pub fn is_blank(s: &str) -> bool {
    for c in s.as_bytes() {
        match *c {
            10 | 13 => return true,
            32 | 9 => (),
            _ => return false,
        }
    }
    true
}

pub fn normalize_reference_label(i: &str) -> String {
    let i = trim_slice(i);
    let mut v = String::with_capacity(i.len());
    let mut last_was_whitespace = false;
    for c in i.chars() {
        for e in c.to_lowercase() {
            if e.is_whitespace() {
                if !last_was_whitespace {
                    last_was_whitespace = true;
                    v.push(' ');
                }
            } else {
                last_was_whitespace = false;
                v.push(e);
            }
        }
    }
    v
}
