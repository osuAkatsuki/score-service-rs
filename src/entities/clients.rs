/// The game clients scores are submitted from, told apart by their user agents.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Client {
    Osu,
    /// circles, a client of its own. It submits the same form as osu!, but with
    /// the score and its client hashes in plain text, and an empty `iv`.
    Circles,
}

impl Client {
    pub fn from_user_agent(user_agent: Option<&str>) -> Option<Self> {
        match user_agent {
            Some("osu!") => Some(Client::Osu),
            Some("circles") => Some(Client::Circles),
            _ => None,
        }
    }
}
