use crate::spinbook::components::classify::{Classification, classify};

pub(crate) trait Classifier {
    fn classify(&self, title: &str) -> String;
}

pub(crate) fn new() -> impl Classifier {
    #[cfg(not(test))]
    {
        ComposedClassifier {}
    }
    #[cfg(test)]
    {
        mocks::MockClassifier {}
    }
}

#[cfg_attr(test, allow(dead_code))]
pub(crate) struct ComposedClassifier;

impl Classifier for ComposedClassifier {
    fn classify(&self, title: &str) -> String {
        match classify(title) {
            Classification::Neutral => "neutral",
            Classification::Urgent => "urgent",
            Classification::Relaxed => "relaxed",
        }.to_string()
    }
}

#[cfg(test)]
mod mocks {
    use super::Classifier;

    pub(crate) struct MockClassifier;

    impl Classifier for MockClassifier {
        fn classify(&self, title: &str) -> String {
            match title.to_lowercase() {
                x if x.contains("asap") => "urgent",
                x if x.contains("someday") => "relaxed",
                _ => "neutral",
            }.to_string()
        }
    }
}
