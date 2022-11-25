use bevy::{asset::Asset, ecs::all_tuples, prelude::*};
use bevy_asset_loader::prelude::{AssetCollection, LoadingState, LoadingStateAppExt};

use super::names::Names;

pub(crate) trait CloneWeak {
    fn clone_weak(&self) -> Self;
}

impl<H: CloneWeak> CloneWeak for Option<H> {
    fn clone_weak(&self) -> Self {
        self.as_ref().map(|h| h.clone_weak())
    }
}

impl<T: Asset> CloneWeak for Handle<T> {
    fn clone_weak(&self) -> Self {
        self.clone_weak()
    }
}

macro_rules! impl_tuple_handle_clone_weak {
    ($($name: ident),*) => {
        impl<$($name: CloneWeak,)*>  CloneWeak for ($($name,)*) {
            #[allow(clippy::unused_unit)]
            fn clone_weak(&self) -> Self {
                #[allow(non_snake_case)]
                let ($($name,)*) = self;
                ($($name.clone_weak(),)*)
            }
        }
    }
}

all_tuples!(impl_tuple_handle_clone_weak, 0, 15, H);

#[derive(Clone, Eq, PartialEq, Debug, Hash)]
pub(crate) enum AllTheLoading {
    Assets,
    Ready,
    Done,
}

pub(crate) struct AssetPlugin;

impl Plugin for AssetPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_state(AllTheLoading::Assets)
            .add_loading_state(
                LoadingState::new(AllTheLoading::Assets)
                    .continue_to_state(AllTheLoading::Ready)
                    .with_collection::<RawUiAssets>()
                    .with_collection::<RawGalaxyAssets>()
                    .with_collection::<ShipAssets>(),
            )
            .add_system_set(SystemSet::on_enter(AllTheLoading::Ready).with_system(done));
    }
}

#[derive(AssetCollection)]
struct RawUiAssets {
    #[asset(path = "ui/dotBlue.png")]
    selection_handle: Handle<Image>,
    #[asset(path = "fonts/StarJediRounded.ttf")]
    font_main: Handle<Font>,
    #[asset(path = "fonts/mandrill.ttf")]
    font_sub: Handle<Font>,
    #[asset(path = "fonts/MaterialIcons-Regular.ttf")]
    font_material: Handle<Font>,
    #[asset(path = "ui/glassPanel_corners.png")]
    panel_texture_handle: Handle<Image>,
    #[asset(path = "ui/glassPanel_cornerBR.png")]
    br_panel_texture_handle: Handle<Image>,
    #[asset(path = "ui/glassPanel_cornerTR.png")]
    tr_panel_texture_handle: Handle<Image>,
    #[asset(path = "ui/glassPanel_cornerTL.png")]
    tl_panel_texture_handle: Handle<Image>,
    #[asset(path = "ui/glassPanel_projection.png")]
    button_texture_handle: Handle<Image>,
}

#[derive(AssetCollection)]
pub(crate) struct ShipAssets {
    #[asset(path = "ships/enemyRed4.png")]
    pub(crate) colony_ship: Handle<Image>,
}

#[derive(Resource)]
pub(crate) struct UiAssets {
    pub(crate) selection_handle: Handle<Image>,
    pub(crate) font_main: Handle<Font>,
    pub(crate) font_sub: Handle<Font>,
    pub(crate) font_material: Handle<Font>,
    pub(crate) panel_handle: (Handle<bevy_ninepatch::NinePatchBuilder<()>>, Handle<Image>),
    pub(crate) br_panel_handle: (Handle<bevy_ninepatch::NinePatchBuilder<()>>, Handle<Image>),
    pub(crate) tr_panel_handle: (Handle<bevy_ninepatch::NinePatchBuilder<()>>, Handle<Image>),
    pub(crate) tl_panel_handle: (Handle<bevy_ninepatch::NinePatchBuilder<()>>, Handle<Image>),
    pub(crate) button_handle: Handle<crate::ui_helper::button::Button>,
}

#[derive(AssetCollection)]
struct RawGalaxyAssets {
    #[asset(path = "star.names")]
    star_names: Handle<Names>,
    #[asset(path = "top-hat.png")]
    hat: Handle<Image>,
}

#[derive(Resource)]
pub(crate) struct GalaxyAssets {
    pub(crate) star_mesh: Handle<Mesh>,
    pub(crate) blue_star: Handle<ColorMaterial>,
    pub(crate) yellow_star: Handle<ColorMaterial>,
    pub(crate) orange_star: Handle<ColorMaterial>,
    pub(crate) unknown: Handle<ColorMaterial>,
    pub(crate) star_names: Handle<Names>,
    pub(crate) hat: Handle<Image>,
}

fn done(world: &mut World) {
    info!("Done Loading Assets");
    unsafe {
        {
            let raw_ui_assets = world.remove_resource_unchecked::<RawUiAssets>().unwrap();
            let mut nine_patches = world
                .get_resource_unchecked_mut::<Assets<bevy_ninepatch::NinePatchBuilder<()>>>()
                .unwrap();
            let mut buttons = world
                .get_resource_unchecked_mut::<Assets<crate::ui_helper::button::Button>>()
                .unwrap();
            let np = bevy_ninepatch::NinePatchBuilder::by_margins(20, 20, 20, 20);
            let panel_handle = (nine_patches.add(np), raw_ui_assets.panel_texture_handle);
            let np = bevy_ninepatch::NinePatchBuilder::by_margins(10, 20, 10, 20);
            let br_panel_handle = (nine_patches.add(np), raw_ui_assets.br_panel_texture_handle);
            let np = bevy_ninepatch::NinePatchBuilder::by_margins(10, 10, 10, 20);
            let tr_panel_handle = (nine_patches.add(np), raw_ui_assets.tr_panel_texture_handle);
            let np = bevy_ninepatch::NinePatchBuilder::by_margins(10, 10, 20, 10);
            let tl_panel_handle = (nine_patches.add(np), raw_ui_assets.tl_panel_texture_handle);
            let button = crate::ui_helper::button::Button::setup(
                &mut nine_patches,
                raw_ui_assets.button_texture_handle,
            );
            let button_handle = buttons.add(button);
            world.insert_resource(UiAssets {
                selection_handle: raw_ui_assets.selection_handle,
                font_main: raw_ui_assets.font_main,
                font_sub: raw_ui_assets.font_sub,
                font_material: raw_ui_assets.font_material,
                panel_handle,
                br_panel_handle,
                tr_panel_handle,
                tl_panel_handle,
                button_handle,
            });
        }

        {
            let raw = world.remove_resource::<RawGalaxyAssets>().unwrap();

            let mut materials = world
                .get_resource_unchecked_mut::<Assets<ColorMaterial>>()
                .unwrap();
            let mut meshes = world.get_resource_unchecked_mut::<Assets<Mesh>>().unwrap();
            #[cfg(not(target_arch = "wasm32"))]
            let over = 1.35;
            #[cfg(target_arch = "wasm32")]
            let over = 1.0;
            world.insert_resource(GalaxyAssets {
                star_mesh: meshes.add(shape::Circle::new(2.5).into()),
                blue_star: materials.add(ColorMaterial::from(Color::rgb(
                    185.0 / 255.0,
                    184.0 / 255.0,
                    over,
                ))),
                yellow_star: materials.add(ColorMaterial::from(Color::rgb(
                    over,
                    over,
                    153.0 / 255.0,
                ))),
                orange_star: materials.add(ColorMaterial::from(Color::rgb(over, 0.5, 0.0))),
                unknown: materials.add(ColorMaterial::from(Color::rgb(0.3, 0.3, 0.3))),
                star_names: raw.star_names,
                hat: raw.hat,
            });
        }
    }
    world
        .resource_mut::<State<AllTheLoading>>()
        .overwrite_set(AllTheLoading::Done)
        .unwrap();
}
