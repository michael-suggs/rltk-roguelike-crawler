#[derive(PartialEq, Clone, Copy)]
pub enum HorizontalPlacement {
    Left,
    Center,
    Right,
}

#[derive(PartialEq, Clone, Copy)]
pub enum VerticalPlacement {
    Top,
    Center,
    Bottom,
}

#[derive(PartialEq, Clone, Copy)]
pub struct PrefabSection {
    pub template: &'static str,
    pub width: usize,
    pub height: usize,
    pub placement: (HorizontalPlacement, VerticalPlacement),
}

pub fn get_template_str(section: PrefabSection) -> String {
    pad(section.template)
}

pub const UNDERGROUND_FORT: PrefabSection = PrefabSection {
    template: RIGHT_FORT,
    width: 15,
    height: 43,
    placement: (HorizontalPlacement::Right, VerticalPlacement::Top),
};

fn pad(template: &str) -> String {
    template
        .split_terminator('\n')
        .map(|line| format!("{: <15}", line))
        .collect::<String>()
}

#[allow(dead_code)]
const RIGHT_FORT: &str = "
     #
  #######
  #     #
  #     #######
  #  g        #
  #     #######
  #     #
  ### ###
    # #
    # #
    # ##
    ^
    ^
    # ##
    # #
    # #
    # #
    # #
  ### ###
  #     #
  #     #
  #  g  #
  #     #
  #     #
  ### ###
    # #
    # #
    # #
    # ##
    ^
    ^
    # ##
    # #
    # #
    # #
  ### ###
  #     #
  #     #######
  #  g        #
  #     #######
  #     #
  #######
     #
";
