use amethyst::{
    assets::{
        AssetStorage, Handle, HotReloadBundle, Loader, PrefabLoader, PrefabLoaderSystem, Processor,
        ProgressCounter,
    },
    core::transform::{Transform, TransformBundle},
    ecs::*,
    input::*,
    prelude::*,
    renderer::{
        Camera, DisplayConfig, DrawShaded, Factory, Hidden, Mesh, MeshCreator, MeshData, Pipeline,
        PosNormTex, Projection, RenderBundle, Renderer, Stage, VertexBuffer,
    },
    ui::*,
    utils::application_root_dir,
};
use amethyst_bsp::{BspFormat, BspPrefabElement};
use gfx::{IndexBuffer, IntoIndexBuffer, Slice};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Default)]
struct LoadMap {
    progress: ProgressCounter,
}

impl SimpleState for LoadMap {
    fn on_start(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        initialize_camera(data.world);

        data.world
            .exec(|loader: PrefabLoader<'_, BspPrefabElement>| {
                loader.load("q3ctf1.bsp", BspFormat, (), &mut self.progress)
            });

        data.world.exec(|mut creator: UiCreator<'_>| {
            creator.create("ui/loading.ron", &mut self.progress);
        });
    }

    fn update(&mut self, data: &mut StateData<'_, GameData<'_, '_>>) -> SimpleTrans {
        use amethyst::assets::Completion;

        match self.progress.complete() {
            Completion::Failed => {
                // TODO: Quit back to menu
                Trans::Quit
            }
            Completion::Complete => Trans::Switch(Box::new(Main)),
            Completion::Loading => Trans::None,
        }
    }
}

struct Main;

impl SimpleState for Main {
    fn on_start(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        initialize_camera(data.world);

        // Done initialising, add the HUD
        data.world.exec(|mut creator: UiCreator<'_>| {
            creator.create("ui/hud.ron", ());
        });
    }
}

fn initialize_camera(world: &mut World) {
    let mut transform = Transform::default();
    transform.translation_mut().z = 1.0;
    world
        .create_entity()
        .with(Camera::from(Projection::perspective(
            1.66,
            std::f32::consts::PI / 2.,
        )))
        .with(transform)
        .build();
}

fn mesh_from_buf_and_indices(
    vertex_buffer: &[PosNormTex],
    index_buffer: impl IntoIterator<Item = u32>,
) -> MeshData {
    MeshData::from(
        index_buffer
            .into_iter()
            .map(|i| vertex_buffer[i as usize])
            .collect::<Vec<_>>(),
    )
}

#[derive(Serialize, Deserialize, Copy, Clone, PartialEq, Eq, Hash)]
enum Axis {
    LookX,
    LookY,
    MoveX,
    MoveY,
}

#[derive(Serialize, Deserialize, Copy, Clone, PartialEq, Eq, Hash)]
enum Action {
    Accept,
    Back,
}

fn main() -> amethyst::Result<()> {
    amethyst::start_logger(Default::default());

    let app_root = PathBuf::from(application_root_dir());
    let resource_dir = app_root.join("resources");

    let game_data = GameDataBuilder::default()
        .with(PrefabLoaderSystem::<BspPrefabElement>::default(), "", &[])
        .with_bundle(TransformBundle::new())?
        .with_bundle(UiBundle::<String, String>::new())?
        .with_bundle(HotReloadBundle::default())?
        .with_bundle(RenderBundle::new(
            Pipeline::build().with_stage(
                Stage::with_backbuffer()
                    .clear_target([0.0, 0.0, 0.0, 1.0], 1.0)
                    .with_pass(DrawShaded::<PosNormTex>::new())
                    .with_pass(DrawUi::new()),
            ),
            Some(DisplayConfig::load(resource_dir.join("display_config.ron"))),
        ))?
        .with_bundle(InputBundle::<String, String>::new())?;

    Application::build(resource_dir, LoadMap::default())?
        .build(game_data)?
        .run();

    Ok(())
}
