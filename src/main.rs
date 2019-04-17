use amethyst::{
    assets::{Directory, HotReloadBundle, PrefabLoader, PrefabLoaderSystem, ProgressCounter},
    core::transform::{Transform, TransformBundle},
    ecs::*,
    input::*,
    prelude::*,
    renderer::{
        Camera, DisplayConfig, DrawShaded, Pipeline, PosNormTex, Projection, RenderBundle, Stage,
    },
    ui::*,
    utils::application_root_dir,
    Error,
};
use amethyst_anysource::AnySource;
use amethyst_bsp::{BspFormat, BspPrefabElement};
use amethyst_pk3::Pk3Source;
use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Default)]
struct LoadMap {
    loading_text: Option<Entity>,
    progress: ProgressCounter,
}

impl SimpleState for LoadMap {
    fn on_start(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        self.loading_text = Some(data.world.exec(|mut creator: UiCreator<'_>| {
            creator.create("ui/loading.ron", &mut self.progress)
        }));

        data.world
            .exec(|loader: PrefabLoader<'_, BspPrefabElement>| {
                loader.load("maps/q3dm0.bsp", BspFormat, (), &mut self.progress)
            });
    }

    fn update(&mut self, data: &mut StateData<'_, GameData<'_, '_>>) -> SimpleTrans {
        use amethyst::assets::Completion;

        match self.progress.complete() {
            Completion::Failed => {
                // TODO: Quit back to menu
                Trans::Quit
            }
            Completion::Complete => {
                if let Some(ent) = &self.loading_text {
                    data.world
                        .entities()
                        .delete(*ent)
                        .expect("Programmer error");
                }

                Trans::Switch(Box::new(Main))
            }
            Completion::Loading => {
                data.world.exec(|mut text: WriteStorage<'_, UiText>| {
                    let percent = (self.progress.num_finished() as f64
                        / self.progress.num_assets() as f64)
                        * 100.;

                    if let Some(text) = text.get_mut(self.loading_text.unwrap()) {
                        text.text = format!("Loading {:.<1}%", percent);
                    }
                });

                Trans::None
            }
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

    transform.translation_mut().x = 346.;
    transform.translation_mut().y = 30.;
    transform.translation_mut().z = 394.;

    world
        .create_entity()
        .with(Camera::from(Projection::perspective(
            1.66,
            std::f32::consts::PI / 2.,
        )))
        .with(transform)
        .build();
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

fn main() -> Result<(), Error> {
    amethyst::start_logger(Default::default());

    let app_root = application_root_dir()?;
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

    let sources: AnySource<_> = AnySource::new().with_source(Directory::new(&resource_dir));

    let sources = sources.with_sources(
        fs::read_dir(resource_dir.join("paks"))?
            .filter_map(|file| {
                let file = match file {
                    Ok(file) => file,
                    Err(e) => return Some(Err(e)),
                };
                let path = file.path();

                if path.extension() != Some(std::ffi::OsStr::new("pk3")) {
                    None
                } else {
                    Some(Pk3Source::open(&path))
                }
            })
            .collect::<Result<Vec<_>, _>>()?,
    );

    Application::build(&resource_dir, LoadMap::default())?
        .with_default_source(sources)
        .build(game_data)?
        .run();

    Ok(())
}
