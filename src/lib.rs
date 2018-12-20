pub fn run() -> std::io::Result<()> {
    Ok(())
}

pub struct Offer {
    pub cpu: f64,
    pub mem: f64,
    pub port: Vec<usize>,
    pub disk: Vec<Disk>,
}

pub struct Disk {
    pub name: String,
    pub volume: f64,
}

pub struct TaskContext {}

impl TaskContext {
    pub fn get_offers(&mut self) -> Vec<Offer> {
        unimplemented!();
    }
}

pub trait Task {
    type Error;

    // fn on_prepare(&mut self, tc: &mut TaskContext) -> Result<(), Self::Error>;
    fn on_running(&mut self, tc: &mut TaskContext) -> Result<(), Self::Error>;
    fn on_done(&mut self, tc: &mut TaskContext) -> Result<(), Self::Error>;
    fn on_fail(&mut self, tc: &mut TaskContext) -> Result<(), Self::Error>;
    fn on_lost(&mut self, tc: &mut TaskContext) -> Result<(), Self::Error>;
}
