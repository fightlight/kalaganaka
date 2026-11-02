use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Task {
    pub id: Uuid,
    pub name: String,
    pub colour: u32,
}

impl std::fmt::Display for Task {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.name)
    }
}
