use crate::compilers::FrontMatter;

pub struct PostCache {
    posts: Vec<FrontMatter>,
}

impl PostCache {
    pub fn new() -> Self {
        let posts = Vec::new();
        Self { posts }
    }

    pub fn add_ref(&mut self, post: FrontMatter) {
        self.posts.push(post);
    }

    pub fn posts(&self) -> &[FrontMatter] {
        self.posts.as_slice()
    }
}
