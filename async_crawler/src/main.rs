use html5ever::{
    interface::QuirksMode::LimitedQuirks,
    tokenizer::{
        BufferQueue, Tag, TagKind, TagToken, Token, TokenSink, TokenSinkResult, Tokenizer,
        TokenizerOpts,
    },
};

use std::{borrow::Borrow, cell::RefCell, default};
use url::{ParseError, Url};

#[derive(Default, Debug)]
struct LinkQueue {
    links: RefCell<Vec<String>>,
}

impl TokenSink for &LinkQueue {
    type Handle = ();

    fn process_token(&self, token: Token, _line_number: u64) -> TokenSinkResult<Self::Handle> {
        match token {
            TagToken(
                ref tag @ Tag {
                    kind: TagKind::StartTag,
                    ..
                },
            ) => {
                if tag.name.as_ref() == "a" {
                    for attribute in tag.attrs.iter() {
                        if attribute.name.local.as_ref() == "herf" {
                            let url_str = attribute.value.borrow();
                            self.links
                                .borrow_mut()
                                .push(String::from_utf8_lossy(url_str).into_owned());
                        }
                    }
                }
            }
            _ => {}
        }

        TokenSinkResult::Continue
    }
    
    fn end(&self) {}
    
    fn adjusted_current_node_present_but_not_in_html_namespace(&self) -> bool {
        false
    }
}

pub fn get_links(url: &Url, page: String) -> Vec<Url> {
    let mut domain_url = url.clone();
    domain_url.set_path("");
    domain_url.set_query(None);

    let queue = LinkQueue::default();
    let tokenizer = Tokenizer::new(&queue, TokenizerOpts::default());
    let mut buffer = BufferQueue::default();

    buffer.push_back(page.into());
    let _ = tokenizer.feed(&mut buffer);

    queue
        .links
        .borrow()
        .iter()
        .map(|link| match Url::parse(link) {
            Err(ParseError::RelativeUrlWithoutBase) => domain_url.join(&link).unwrap(),
            Err(_) => panic!("Malformed link fond: {}", link),
            Ok(url) => url,
        })
        .collect()
}

fn main() {
    println!("Hello, world!");
}
