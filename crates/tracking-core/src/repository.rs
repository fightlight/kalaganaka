use crate::Snapshot;

/// Storage implementations live outside the domain crate.
pub trait Repository {
    type Error;

    fn load(&self) -> Result<Snapshot, Self::Error>;
    fn save(&mut self, snapshot: &Snapshot) -> Result<(), Self::Error>;
}
