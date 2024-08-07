use crate::*;

pub struct AssetPath;
impl AssetPath {
    pub const FONT_LIGHT: &'static str = "fonts/rajdhani/Rajdhani-Light.ttf";
    pub const FONT_REGULAR: &'static str = "fonts/rajdhani/Rajdhani-Regular.ttf";
    pub const FONT_MEDIUM: &'static str = "fonts/rajdhani/Rajdhani-Medium.ttf";
    pub const FONT_SEMIBOLD: &'static str = "fonts/rajdhani/Rajdhani-SemiBold.ttf";
    pub const FONT_BOLD: &'static str = "fonts/rajdhani/Rajdhani-Bold.ttf";

    pub const CURSOR: &'static str = "images/cursor.png";

    pub const ACID_SPRAYER: &'static str = "turrets/AcidSprayer.png";
    pub const PLASMA_RAY: &'static str = "turrets/PlasmaRay.png";
    pub const PULSE_BLASTER: &'static str = "turrets/PulseBlaster.png";
    pub const RAIL_GUN: &'static str = "turrets/RailGun.png";

    pub const CHEVRON_LEFT: &'static str = "images/chevron_left.png";
    pub const CHEVRON_RIGHT: &'static str = "images/chevron_right.png";
}

pub trait BevypunkColorPalette {
    const BEVYPUNK_RED: Color;
    const BEVYPUNK_RED_DIM: Color;
    const BEVYPUNK_YELLOW: Color;
    const BEVYPUNK_BLUE: Color;
}
impl BevypunkColorPalette for Color {
    const BEVYPUNK_RED: Color = Color::srgba(255./255., 98./255., 81./255., 1.0);
    const BEVYPUNK_RED_DIM: Color = Color::srgba(172./255., 64./255., 63./255., 1.0);
    const BEVYPUNK_YELLOW: Color = Color::linear_rgba(252./255., 226./255., 8./255., 1.0);
    const BEVYPUNK_BLUE: Color = Color::srgba(8./255., 226./255., 252./255., 1.0);
}
