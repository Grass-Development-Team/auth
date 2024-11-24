use std::collections::BTreeMap;
use std::error::Error;
use std::fmt::Debug;
use tracing::field::Field;

pub struct Visitor {
    pub fields: BTreeMap<String, String>,
}

impl Visitor {
    pub fn new() -> Self {
        Visitor {
            fields: BTreeMap::new(),
        }
    }

    pub fn message(&self) -> String {
        let mut tmp = String::new();
        if self.fields.contains_key("message") {
            tmp = self.fields["message"].to_string();
        }
        if self.fields.len() > 1 {
            tmp += "\n\t";
            let mut space = false;
            for (k, v) in self.fields.iter() {
                if k == "message" {
                    continue;
                }
                tmp += format!("{}{} = {}", if space { " " } else { "" }, k, v).as_str();
                if !space {
                    space = true;
                }
            }
        }
        tmp
    }
}

impl tracing::field::Visit for Visitor {
    fn record_f64(&mut self, field: &Field, value: f64) {
        self.fields
            .insert(field.name().to_string(), format!("{}", value));
    }

    fn record_i64(&mut self, field: &Field, value: i64) {
        self.fields
            .insert(field.name().to_string(), format!("{}", value));
    }

    fn record_u64(&mut self, field: &Field, value: u64) {
        self.fields
            .insert(field.name().to_string(), format!("{}", value));
    }

    fn record_i128(&mut self, field: &Field, value: i128) {
        self.fields
            .insert(field.name().to_string(), format!("{}", value));
    }

    fn record_u128(&mut self, field: &Field, value: u128) {
        self.fields
            .insert(field.name().to_string(), format!("{}", value));
    }

    fn record_bool(&mut self, field: &Field, value: bool) {
        self.fields
            .insert(field.name().to_string(), format!("{}", value));
    }

    fn record_str(&mut self, field: &Field, value: &str) {
        self.fields
            .insert(field.name().to_string(), value.to_string());
    }

    fn record_error(&mut self, field: &Field, value: &(dyn Error + 'static)) {
        self.fields
            .insert(field.name().to_string(), format!("{}", value));
    }

    fn record_debug(&mut self, field: &Field, value: &dyn Debug) {
        self.fields
            .insert(field.name().to_string(), format!("{:?}", value));
    }
}
