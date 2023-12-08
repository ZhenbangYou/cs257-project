use std::collections::BTreeSet;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KeyRule {
    Identity,
    Fixed(String),
    IdWithPrefix(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InputCond {
    Always,
    MatchesKey(String),
    MatchesKeyValue(String, String),
}

#[derive(Debug)]
pub struct OutputSchema {
    pub fixed_keys: Vec<String>,
    pub dynamic_keys: Vec<(KeyRule, InputCond)>,
}

pub struct OutputSchemaBuilder {
    fixed_keys: BTreeSet<String>,
    dynamic_keys: Vec<(KeyRule, InputCond)>,
}

impl OutputSchemaBuilder {
    pub fn add_fixed(mut self, key: impl Into<String>) -> Self {
        self.fixed_keys.insert(key.into());
        self
    }

    pub fn add_rule_for_every_input(mut self, key: KeyRule, cond: InputCond) -> Self {
        self.dynamic_keys.push((key, cond));
        self
    }

    pub fn carry_all(self) -> Self {
        self.add_rule_for_every_input(KeyRule::Identity, InputCond::Always)
    }

    pub fn build(self) -> OutputSchema {
        OutputSchema {
            fixed_keys: self.fixed_keys.into_iter().collect(),
            dynamic_keys: self.dynamic_keys,
        }
    }
}

impl OutputSchema {
    pub fn new() -> OutputSchemaBuilder {
        OutputSchemaBuilder {
            fixed_keys: Default::default(),
            dynamic_keys: Default::default(),
        }
    }

    pub fn fixed_keys(&self) -> impl Iterator<Item = &str> {
        self.fixed_keys.iter().map(|s| s.as_str())
    }

    pub fn dynamic_keys(&self) -> &[(KeyRule, InputCond)] {
        &self.dynamic_keys
    }
}
