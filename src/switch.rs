use tegra_rcm::Switch;

#[derive(Debug, Clone)]
pub enum SwitchData {
    Available(Switch),
    None,
    Done,
}

impl PartialEq for SwitchData {
    fn eq(&self, other: &Self) -> bool {
        match self {
            SwitchData::Available(_) => matches!(other, SwitchData::Available(_)),
            SwitchData::None => matches!(other, SwitchData::None),
            SwitchData::Done => matches!(other, SwitchData::Done),
        }
    }
}
