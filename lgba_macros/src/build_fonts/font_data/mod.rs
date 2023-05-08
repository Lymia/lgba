const UNSCII_DATA: &str = include_str!("unscii-8.hex");
const MISAKI_DATA: &str = include_str!("misaki_gothic_2nd.hex");

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Debug, Hash)]
pub struct CharacterInfo {
    pub ch: char,
    pub data: u64,
    pub is_half_width: bool,
}
impl CharacterInfo {
    fn new(ch: char, data: u64) -> Self {
        let has_left = (data & 0xF0F0F0F0F0F0F0F0) != 0;
        let has_right = (data & 0x0F0F0F0F0F0F0F0F) != 0;
        let is_half_width = has_left && !has_right;
        CharacterInfo { ch, data, is_half_width }
    }
}

pub fn data_is_half_width(data: u64) -> bool {
    (data & 0x0F0F0F0F0F0F0F0F) == 0
}

pub struct CharacterSets {
    pub unscii: Vec<CharacterInfo>,
    pub misaki: Vec<CharacterInfo>,
}

fn parse_hex(chars: &mut Vec<CharacterInfo>, data: &str) {
    for line in data.split('\n').filter(|x| !x.is_empty()) {
        let mut split = line.split(':');

        let hex_str = split.next().unwrap();
        let hex_bmp = split.next().unwrap();

        let ch = char::from_u32(u32::from_str_radix(&hex_str, 16).unwrap()).unwrap();
        let data = u64::from_str_radix(&hex_bmp, 16).unwrap();
        if ch != '\0' {
            chars.push(CharacterInfo::new(ch, data));
        }
    }
    chars.sort_by_key(|x| x.ch as u32);
}

pub fn load_fonts() -> CharacterSets {
    let mut characters = CharacterSets { unscii: Vec::new(), misaki: Vec::new() };
    parse_hex(&mut characters.unscii, UNSCII_DATA);
    parse_hex(&mut characters.misaki, MISAKI_DATA);
    characters
}
