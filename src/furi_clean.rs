use clipboard_script::{is_jp};

use clipboard_master::{Master, ClipboardHandler, CallbackResult};
use clipboard_win::Clipboard;

use std::io;

#[inline(always)]
///Returns whether text contains only JP kana.
pub fn is_furi_skip<T: AsRef<str>>(text: T) -> bool {
    let text = text.as_ref();
    text.chars().all(|elem_char| match elem_char { '﹅'| ' ' | //Special case for tons of ````
                                                   '\u{3040}'..='\u{309f}'| //hiragana
                                                   '\u{30a0}'..='\u{30ff}' //katakana
                                                      => true,
                                                   _  => false,
    })
}

struct Handler;

impl ClipboardHandler for Handler {
    fn on_clipboard_change(&mut self) -> CallbackResult {
        const SPLIT_PAT: &[char] = &['\r', '\n'];

        let mut attempts = 10;
        let clip = loop {
            match Clipboard::new() {
                Ok(clip) => break clip,
                Err(error) => {
                    attempts -= 1;

                    match attempts {
                        0 => {
                            eprintln!("Failed to open clipboard within 10 attempts. Error: {}", error);
                            return CallbackResult::Next;
                        },
                        _ => continue,
                    }
                }
            }
        };

        let content = match clip.get_string() {
            Ok(content) => content,
            Err(_) => return CallbackResult::Next,
        };

        if !is_jp(&content) || !content.contains(SPLIT_PAT) {
            return CallbackResult::Next;
        }

        let text = content.trim();
        let text_len = content.len();

        let mut new_text = String::with_capacity((text_len + text_len) / 3);

        let parts = text.split(SPLIT_PAT).map(|part| part.trim()).collect::<Vec<_>>();

        new_text.push_str(unsafe { parts.get_unchecked(0) });

        for idx in 1..parts.len()-1 {
            let part = unsafe { parts.get_unchecked(idx) };

            if part.len() == 0 || is_furi_skip(part) {
                continue;
            }

            new_text.push_str(part);
        }

        new_text.push_str(unsafe { parts.get_unchecked(parts.len()-1) });

        if text_len != new_text.len() {
            let _ = clip.set_string(&new_text);
        }

        CallbackResult::Next
    }

    fn on_clipboard_error(&mut self, error: io::Error) -> CallbackResult {
        eprintln!("Error: {}", error);
        CallbackResult::Next
    }
}

fn main() {
    let _ = Master::new(Handler).run();
}
