use crate::Position;

#[derive(PartialEq, Clone, Copy)]
pub struct PrefabRoom {
    pub template: &'static str,
    pub width: usize,
    pub height: usize,
    pub first_depth: i32,
    pub last_depth: i32,
}

pub fn get_template_str(room: PrefabRoom) -> String {
    room.template
        .split_terminator('\n')
        .map(|line| format!("{line: <0$}", room.width, line = line))
        .collect::<String>()
}

pub struct Vault<'a> {
    vault: &'a PrefabRoom,
    pos: Position,
}

pub const NOT_A_TRAP: PrefabRoom = PrefabRoom {
    template: NOT_A_TRAP_MAP,
    width: 5,
    height: 5,
    first_depth: 0,
    last_depth: 100,
};

pub const SILLY_SMILE: PrefabRoom = PrefabRoom {
    template: SILLY_SMILE_MAP,
    width: 6,
    height: 6,
    first_depth: 0,
    last_depth: 100,
};

pub const CHECKERBOARD: PrefabRoom = PrefabRoom {
    template: CHECKERBOARD_MAP,
    width: 6,
    height: 6,
    first_depth: 0,
    last_depth: 100,
};

const NOT_A_TRAP_MAP: &str = "

 ^^^
 ^!^
 ^^^

";

const SILLY_SMILE_MAP: &str = "

 ^  ^
  #

 ###

";

const CHECKERBOARD_MAP: &str = "

 g#%#
 #!#
 ^# #

";
