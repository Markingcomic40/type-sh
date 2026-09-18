/// What ends a test.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Limit {
    Time(u64),
    Words(u64),
    None,
}

#[derive(Clone, Debug)]
pub struct TestConfig {
    pub limit: Limit,
    pub wordlist: String,
    pub freedom: bool,
}
