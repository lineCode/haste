

pub fn fetch_offer() -> Vec<Offer> {
    unimplemented!()
}

/// server acquire resource -> by using offer
/// agent report Offer by using offer
#[derive(Clone, Debug)]
pub struct Offer {
    pub host: String,
    // CPU is the a percentage value.
    pub cpu: usize,
    pub memory: usize,
    pub ports: Vec<usize>,
}
