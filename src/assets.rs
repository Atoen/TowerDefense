use crate::*;

pub struct AssetPath;
impl AssetPath {
    pub const FONT_LIGHT: &'static str = "fonts/rajdhani/Rajdhani-Light.ttf";
    pub const FONT_REGULAR: &'static str = "fonts/rajdhani/Rajdhani-Regular.ttf";
    pub const FONT_MEDIUM: &'static str = "fonts/rajdhani/Rajdhani-Medium.ttf";
    pub const FONT_SEMIBOLD: &'static str = "fonts/rajdhani/Rajdhani-SemiBold.ttf";
    pub const FONT_BOLD: &'static str = "fonts/rajdhani/Rajdhani-Bold.ttf";

    pub const CURSOR: &'static str = "images/cursor.png";
    pub const PAUSE: &'static str = "images/pause_icon.png";

    pub const ACID_SPRAYER: &'static str = "turrets/AcidSprayer.png";
    pub const PLASMA_RAY: &'static str = "turrets/PlasmaRay.png";
    pub const PULSE_BLASTER: &'static str = "turrets/PulseBlaster.png";
    pub const RAIL_GUN: &'static str = "turrets/RailGun.png";

    pub const ARROW_LEFT: &'static str = "images/arrow_left.png";
    pub const ARROW_RIGHT: &'static str = "images/arrow_right.png";

    pub const NEBULA: &'static str = "images/nebula.png";
    pub const GRID_CELL: &'static str = "images/grid_cell.png";
}

#[allow(dead_code)]
pub trait ColorPalette {
    const RED: Color;
    const RED_DIM: Color;
    const YELLOW: Color;
    const BLUE: Color;
    const GREEN: Color;
    const ORANGE: Color;
    const PURPLE: Color;
    
    const GRAY_100: Color;
    const GRAY_200: Color;
    const GRAY_300: Color;
    const GRAY_400: Color;
    const GRAY_500: Color;
    const GRAY_600: Color;
    const GRAY_700: Color;
    const GRAY_800: Color;
    const GRAY_900: Color;
}

impl ColorPalette for Color {
    const RED: Color = Color::srgb(1., 98./255., 81./255.);
    const RED_DIM: Color = Color::srgb(172./255., 64./255., 63./255.);
    const YELLOW: Color = Color::srgb(252./255., 226./255., 8./255.);
    const BLUE: Color = Color::srgb(8./255., 226./255., 252./255.);
    const GREEN: Color = Color::srgb(81./255., 1., 98./255.);
    const ORANGE: Color = Color::srgb(1., 165./255., 0./255.);
    const PURPLE: Color = Color::srgb(128./255., 0./255., 128./255.);
    
    const GRAY_100: Color = Color::srgb(248./255., 249./255., 250./255.);
    const GRAY_200: Color = Color::srgb(233./255., 236./255., 239./255.);
    const GRAY_300: Color = Color::srgb(222./255., 226./255., 230./255.);
    const GRAY_400: Color = Color::srgb(206./255., 212./255., 218./255.);
    const GRAY_500: Color = Color::srgb(173./255., 181./255., 189./255.);
    const GRAY_600: Color = Color::srgb(108./255., 117./255., 125./255.);
    const GRAY_700: Color = Color::srgb(73./255., 80./255., 87./255.);
    const GRAY_800: Color = Color::srgb(52./255., 58./255., 64./255.);
    const GRAY_900: Color = Color::srgb(33./255., 37./255., 41./255.);
}