use crate::spinbook::components::classify::{Classification, classify};

pub(crate) trait ClassificationService {
    fn classify(&self, title: &str) -> String;
}

pub(crate) fn new() -> impl ClassificationService {
    #[cfg(not(test))]
    {
    ComposedClassificationService {}
    }
    #[cfg(test)]
    {
        mocks::MockClassificationService {}
    }
 }

pub(crate) struct ComposedClassificationService;

impl ClassificationService for ComposedClassificationService {
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
    use super::ClassificationService;

    pub(crate) struct MockClassificationService;

    impl ClassificationService for MockClassificationService {
        fn classify(&self, title: &str) -> String {
            match title.to_lowercase() {
                x if x.contains("asap") => "urgent",
                x if x.contains("someday") => "relaxed",
                _ => "neutral",
            }.to_string()
        }
    }
}