use crate::*;

pub struct AssetPath;

#[allow(dead_code)]
impl AssetPath {
    const CURSOR: &'static str = "images/cursor.png";
    const PAUSE: &'static str = "images/pause_icon.png";

    const ACID_SPRAYER: &'static str = "turrets/AcidSprayer.png";
    const PLASMA_RAY: &'static str = "turrets/PlasmaRay.png";
    const PULSE_BLASTER: &'static str = "turrets/PulseBlaster.png";
    const RAIL_GUN: &'static str = "turrets/RailGun.png";

    const ARROW_LEFT: &'static str = "images/arrow_left.png";
    const ARROW_RIGHT: &'static str = "images/arrow_right.png";

    const NEBULA: &'static str = "images/nebula.png";
    const GRID_CELL: &'static str = "images/grid_cell.png";

    const SELECTED_CELL: &'static str = "images/selected_cell.png";
    const MODULE: &'static str = "images/module.png";
    const MINE_SHEET: &'static str = "images/mine_sheet.png";
    const BLADES_BIG: &'static str = "images/blade_big.png";
    const BLADES_SMALL: &'static str = "images/blade_small.png";
}

#[derive(Resource)]
pub struct UiTextures {
    pub arrow_left: Handle<Image>,
    pub arrow_right: Handle<Image>,
    pub nebula: Handle<Image>,
    pub pause: Handle<Image>
}

impl UiTextures {
    pub fn load(asset_server: &AssetServer) -> Self {
        Self {
            arrow_right: asset_server.load(AssetPath::ARROW_RIGHT),
            arrow_left: asset_server.load(AssetPath::ARROW_LEFT),
            nebula: asset_server.load(AssetPath::NEBULA),
            pause: asset_server.load(AssetPath::PAUSE)
        }
    }
}

#[derive(Resource)]
pub struct GameTextures {
    pub selected_cell: Handle<Image>,
    pub module: Handle<Image>,
    pub mine_texture: Handle<Image>,
    pub mine_atlas: Handle<TextureAtlasLayout>,
    pub blades_big: Handle<Image>,
    pub blades_small: Handle<Image>,

    pub acid_sprayer: Handle<Image>,
    pub plasma_ray: Handle<Image>,
    pub pulse_blaster: Handle<Image>,
    pub rail_gun: Handle<Image>
}

impl GameTextures {
    pub fn load(asset_server: &AssetServer, mut texture_atlases: ResMut<Assets<TextureAtlasLayout>>) -> Self {

        let mine_sheet_handle = asset_server.load(AssetPath::MINE_SHEET);
        let mine_sheet = TextureAtlasLayout::from_grid(UVec2::new(100, 100), 2, 1, None, None);
        let mine_layout = texture_atlases.add(mine_sheet);

        Self {
            selected_cell: asset_server.load(AssetPath::SELECTED_CELL),
            module: asset_server.load(AssetPath::MODULE),
            blades_big: asset_server.load(AssetPath::BLADES_BIG),
            blades_small: asset_server.load(AssetPath::BLADES_SMALL),
            mine_atlas: mine_layout,
            mine_texture: mine_sheet_handle,

            acid_sprayer: asset_server.load(AssetPath::ACID_SPRAYER),
            plasma_ray: asset_server.load(AssetPath::PLASMA_RAY),
            pulse_blaster: asset_server.load(AssetPath::PULSE_BLASTER),
            rail_gun: asset_server.load(AssetPath::RAIL_GUN)
        }
    }
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