

pub fn acquire_offer() -> Vec<Offer> {
    unimplemented!()
}

/// server acquire resource -> by using offer
/// agent report Offer by using offer
#[derive(Clone, Debug)]
pub struct Offer {
    host: String,
    cpu: f64,
    memory: f64,
    ports: Vec<u32>,
}
