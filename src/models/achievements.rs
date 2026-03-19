use crate::entities::mode::Gamemode;

#[derive(Debug)]
pub struct Achievement {
    pub id: i32,
    pub icon_file: String,
    pub name: String,
    pub description: String,
    pub hidden: bool,
    pub mode: Option<Gamemode>,
}

impl Achievement {
    pub fn osu_format(&self) -> String {
        format!("{}+{}+{}", self.icon_file, self.name, self.description)
    }
}
