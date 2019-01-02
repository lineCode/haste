use crate::proto::{Action, SystemdAction};
use failure::{Error, Fail};

pub fn do_action(action: SystemdAction, port: i64) -> Result<(), Error> {
    unimplemented!()
}
