use amethyst;
use amethyst::ecs::Entity;
use amethyst::prelude::*;
use amethyst::renderer::{Event, KeyboardInput, ScreenDimensions, VirtualKeyCode, WindowEvent};
use amethyst::ui::{FontHandle, UiResize, UiText, UiTransform};

const FONT_SIZE: f32 = 17.;

/// `State` where game play takes place.
///
/// Current implementation is a place holder until this is properly developed.
#[derive(Debug, Default)]
pub struct State {
    /// Holds the info label.
    entity: Option<Entity>,
}

impl State {
    /// Returns a new `game_play::State`.
    pub fn new() -> Self {
        Default::default()
    }

    fn initialize_informative(&mut self, world: &mut World) {
        let font = read_font(world);

        let mut text_transform = UiTransform::new("info".to_string(), 20., 20., 1., 400., 100., 0);
        let ui_text_size_fn = |_transform: &mut UiTransform, (_width, _height)| {};

        {
            let dim = world.read_resource::<ScreenDimensions>();
            ui_text_size_fn(&mut text_transform, (dim.width(), dim.height()));
        }

        let info_entity = world
            .create_entity()
            .with(text_transform)
            .with(UiText::new(
                font,
                "Press [Escape] to return to the previous menu.".to_string(),
                [1., 1., 1., 1.],
                FONT_SIZE,
            ))
            .with(UiResize(Box::new(ui_text_size_fn)))
            .build();

        self.entity.get_or_insert(info_entity);
    }

    fn terminate_informative(&mut self, world: &mut World) {
        world
            .delete_entity(self.entity.take().expect("Expected entity to be set."))
            .expect("Failed to delete entity.");
    }
}

impl amethyst::State for State {
    fn on_start(&mut self, world: &mut World) {
        self.initialize_informative(world);
    }

    fn handle_event(&mut self, _: &mut World, event: Event) -> Trans {
        match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::KeyboardInput {
                    input:
                        KeyboardInput {
                            virtual_keycode: Some(VirtualKeyCode::Escape),
                            ..
                        },
                    ..
                } => {
                    info!("Returning from `game_play::State`.");
                    Trans::Pop
                }
                _ => Trans::None,
            },
            _ => Trans::None,
        }
    }

    fn on_stop(&mut self, world: &mut World) {
        self.terminate_informative(world);
    }
}

fn read_font(world: &mut World) -> FontHandle {
    use application_ui::FontVariant::Regular;
    world
        .read_resource_with_id::<FontHandle>(Regular.into())
        .clone()
}