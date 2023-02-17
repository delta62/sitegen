use chrono::{DateTime, Utc};
use serde::Serialize;

#[derive(Serialize)]
pub struct PostRef {
    title: String,
    publish_date: Option<DateTime<Utc>>,
    href: String,
    intro: String,
}

#[derive(serde::Serialize)]
pub struct PostData {
    pub body: String,
    pub intro: String,
    pub publish_date: Option<DateTime<Utc>>,
    pub slug: String,
    pub title: String,
}

impl From<PostData> for PostRef {
    fn from(value: PostData) -> Self {
        Self {
            title: value.title.clone(),
            publish_date: value.publish_date,
            intro: value.intro.clone(),
            href: format!("{}.html", value.slug),
        }
    }
}

pub struct PostCache {
    posts: Vec<PostRef>,
}

impl PostCache {
    pub fn new() -> Self {
        let posts = Vec::new();
        Self { posts }
    }

    pub fn add_ref(&mut self, post: PostRef) {
        self.posts.push(post);
    }

    pub fn posts(&self) -> &[PostRef] {
        self.posts.as_slice()
    }
}
