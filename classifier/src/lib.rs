use crate::bindings::exports::spinbook::components::classify::{Classification, Guest};

mod bindings {
    wit_bindgen::generate!({
        path: "wit/world.wit",
    });
    use super::ClassifierComponent;
    export!(ClassifierComponent);
}

struct ClassifierComponent;

impl Guest for ClassifierComponent {
    #[allow(async_fn_in_trait)]
    fn classify(input: String) -> Classification {
        match input.to_lowercase() {
            x if x.contains("urgent") || x.contains("asap") || x.contains("immediately") => {
                Classification::Urgent
            }
            x if x.contains("later") || x.contains("whenever") || x.contains("someday") => {
                Classification::Relaxed
            }
            _ => Classification::Neutral,
        }
    }
}
