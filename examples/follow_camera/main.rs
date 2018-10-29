//! Demonstrates sprite z ordering
//!
//! Sprites are originally from <https://opengameart.org/content/bat-32x32>, edited to show
//! layering and blending.

extern crate amethyst;
#[macro_use]
extern crate log;

mod png_loader;
mod sprite;
mod sprite_sheet_loader;

use amethyst::{
    assets::{AssetStorage, Loader},
    core::{
        cgmath::{Matrix4, Vector3},
        transform::{GlobalTransform, Transform, TransformBundle},
        Parent,
    },
    ecs::prelude::Entity,
    prelude::*,
    renderer::{
        Camera, DisplayConfig, DrawSprite, MaterialTextureSet, Pipeline, Projection, RenderBundle,
        SpriteRender, SpriteSheet, SpriteSheetHandle, Stage,
    },
    utils::application_root_dir,
};

use sprite::SpriteSheetDefinition;

#[derive(Debug, Clone)]
struct LoadedSpriteSheet {
    sprite_sheet_handle: SpriteSheetHandle,
    sprite_count: usize,
    sprite_w: f32,
    sprite_h: f32,
}

#[derive(Debug, Default)]
struct Example {
    camera: Option<Entity>,
    entities: Vec<Entity>,
    loaded_sprite_sheet: Option<LoadedSpriteSheet>,
}

impl Example {
    fn new() -> Self {
        Example {
            camera: None,
            entities: Vec::new(),
            loaded_sprite_sheet: None,
        }
    }
}

impl<'a, 'b> SimpleState<'a, 'b> for Example {
    fn on_start(&mut self, data: StateData<GameData>) {
        let StateData { world, .. } = data;
        self.loaded_sprite_sheet = Some(load_sprite_sheet(world));
        let player_entity = world
            .create_entity()
            .with(SpriteRender {
                flip_vertical: false,
                flip_horizontal: false,
                sprite_number: 0,
                sprite_sheet: self
                    .loaded_sprite_sheet
                    .clone()
                    .unwrap()
                    .sprite_sheet_handle,
            })
            .with(GlobalTransform::default())
            .with(Transform::default())
            .build();
        world
            .create_entity()
            .with(Camera::from(Projection::orthographic(
                0.0, 100.0, 100.0, 0.0,
            )))
            .with(GlobalTransform(
                Matrix4::from_translation(Vector3::new(0.0, 0.0, 1.0)).into(),
            ))
            .with(Parent {
                entity: player_entity,
            })
            .with(Transform::default())
            .build();
    }
}

/// Loads and returns a handle to a sprite sheet.
///
/// The sprite sheet consists of two parts:
///
/// * texture: the pixel data
/// * `SpriteSheet`: the layout information of the sprites on the image
fn load_sprite_sheet(world: &mut World) -> LoadedSpriteSheet {
    let sprite_sheet_index = 0;

    // Store texture in the world's `MaterialTextureSet` resource (singleton hash map)
    // This is used by the `DrawSprite` pass to look up the texture from the `SpriteSheet`
    let texture = png_loader::load("texture/bat_semi_transparent.png", world);
    world
        .write_resource::<MaterialTextureSet>()
        .insert(sprite_sheet_index, texture);

    let sprite_w = 32.;
    let sprite_h = 32.;
    let sprite_sheet_definition = SpriteSheetDefinition::new(sprite_w, sprite_h, 2, 6, false);

    let sprite_sheet = sprite_sheet_loader::load(sprite_sheet_index, &sprite_sheet_definition);
    let sprite_count = sprite_sheet.sprites.len();

    let sprite_sheet_handle = {
        let loader = world.read_resource::<Loader>();
        loader.load_from_data(
            sprite_sheet,
            (),
            &world.read_resource::<AssetStorage<SpriteSheet>>(),
        )
    };

    LoadedSpriteSheet {
        sprite_sheet_handle,
        sprite_count,
        sprite_w,
        sprite_h,
    }
}

fn main() -> amethyst::Result<()> {
    amethyst::start_logger(Default::default());

    let app_root = application_root_dir();

    let display_config = DisplayConfig::load(format!(
        "{}/examples/follow_camera/resources/display_config.ron",
        app_root
    ));

    let pipe = Pipeline::build().with_stage(
        Stage::with_backbuffer()
            .clear_target([0., 0., 0., 1.], 5.)
            .with_pass(DrawSprite::new()),
    );

    let assets_directory = format!("{}/examples/assets/", app_root);

    let game_data = GameDataBuilder::default()
        .with_bundle(TransformBundle::new())?
        .with_bundle(RenderBundle::new(pipe, Some(display_config)).with_sprite_sheet_processor())?;

    let mut game = Application::new(assets_directory, Example::new(), game_data)?;
    game.run();

    Ok(())
}
