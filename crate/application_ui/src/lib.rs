#![deny(missing_docs)]
#![deny(missing_debug_implementations)]

//! Handles resources common to an application's UI.
//!
//! Currently this just registers fonts with the world. In the future, this crate may also handle
//! switching between themes and internationalization.
//!
//! # Usage
//!
//! ## Font Configuration
//!
//! This bundle expects to find a `resources/font_config.ron` file next to the executable. The
//! configuration format is as follows:
//!
//! ```rust,ignore
//! (
//!     regular: "relative/path/to/regular.ttf",
//!     bold: "relative/path/to/bold.ttf",
//!     italic: "relative/path/to/italic.ttf",
//!     bold_italic: "relative/path/to/bold_italic.ttf",
//! )
//! ```
//!
//! The paths are relative to the `assets` directory next to the executable. Visually, the directory
//! structure is as follows:
//!
//! ```text
//! bin
//! ├── resources
//! │  ├── font_config.ron
//! │  └── ...
//! ├── assets
//! │   └── relative
//! │      └── path
//! │         ├── to
//! │         │  ├── regular.ttf
//! │         │  ├── bold.ttf
//! │         │  ├── it.ttf
//! │         │  └── boldit.ttf
//! │         └── ...
//! ├── my_app.exe
//! └── ...
//! ```
//!
//! ## Code
//!
//! This section explains the parts of the code to render text using the loaded fonts.
//!
//! ```rust,no_run
//! // === Imports === //
//! extern crate amethyst;
//! #[macro_use]
//! extern crate application;
//! extern crate application_ui;
//!
//! // Uncomment the next line when the `state` module is in a separate file. For documentation
//! // purposes, this is placed in the same code block.
//! // ---
//! // mod state;
//!
//! use std::process;
//!
//! use amethyst::prelude::*;
//! use amethyst::renderer::{DisplayConfig, Pipeline, RenderBundle, Stage};
//! use amethyst::ui::{DrawUi, UiBundle};
//! use application::config::find_in;
//! use application_ui::ApplicationUiBundle;
//!
//! use state::TextState;
//!
//! // === Running the Amethyst application === //
//! fn run() -> Result<(), amethyst::Error> {
//!     let display_config = DisplayConfig::load(
//!         find_in(
//!             "resources",
//!             "display_config.ron",
//!             Some(development_base_dirs!()),
//!         ).unwrap(),
//!     );
//!
//!     // Make sure your `Pipeline` has the `DrawUi` pass
//!     let pipe = Pipeline::build().with_stage(
//!         Stage::with_backbuffer()
//!             .clear_target([0.1, 0.1, 0.1, 1.], 1.)
//!             .with_pass(DrawUi::new()),
//!     );
//!
//!     // Make sure the `UiBundle` is added before the `ApplicationUiBundle` as it sets up the
//!     // `AssetStorage<FontAsset>` and `Loader` needed to load the fonts.
//!     let mut app = Application::build("assets", TextState)?
//!         .with_bundle(UiBundle::new())?
//!         .with_bundle(RenderBundle::new(pipe, Some(display_config)))?
//!         .with_bundle(ApplicationUiBundle::new())?
//!         .build()
//!         .expect("Failed to build application.");
//!
//!     app.run();
//!
//!     Ok(())
//! }
//!
//! fn main() {
//!     if let Err(e) = run() {
//!         println!("Failed to execute example: {}", e);
//!         process::exit(1);
//!     }
//! }
//!
//! // We recommend that this be in a separate file.
//! mod state {
//!     use amethyst::prelude::*;
//!     use amethyst::renderer::{Event, KeyboardInput, ScreenDimensions, VirtualKeyCode,
//!                              WindowEvent};
//!     use amethyst::ui::{FontHandle, UiResize, UiText, UiTransform};
//!
//!     const FONT_SIZE: f32 = 25.;
//!
//!     pub struct TextState;
//!
//!     impl State for TextState {
//!         // When managing entities with text components, it's best to set them up when the state
//!         // is being initialized.
//!         fn on_start(&mut self, world: &mut World) {
//!             initialize_text(world);
//!         }
//! #
//! #         fn handle_event(&mut self, _: &mut World, event: Event) -> Trans {
//! #             match event {
//! #                 Event::WindowEvent { event, .. } => match event {
//! #                     WindowEvent::KeyboardInput {
//! #                         input:
//! #                             KeyboardInput {
//! #                                 virtual_keycode: Some(VirtualKeyCode::Escape),
//! #                                 ..
//! #                             },
//! #                         ..
//! #                     } => Trans::Quit,
//! #                     _ => Trans::None,
//! #                 },
//! #                 _ => Trans::None,
//! #             }
//! #         }
//! #     }
//!
//!     fn initialize_text(world: &mut World) {
//!         let (font_regular, font_bold, font_italic, font_bold_italic) = read_fonts(world);
//!
//!         let mut fonts = vec![
//!             // font, text to display, y_offset
//!             (font_regular, "regular", 0.),
//!             (font_bold, "bold", 50.),
//!             (font_italic, "italic", 100.),
//!             (font_bold_italic, "bold_italic", 150.),
//!         ];
//!
//!         fonts.drain(..).for_each(|(font, text, y_offset)| {
//!             let mut text_transform =
//!                 UiTransform::new(text.to_string(), 20., y_offset + 20., 1., 400., 100., 0);
//!             let ui_text_size_fn = |_transform: &mut UiTransform, (_width, _height)| {};
//!
//!             // Adjust the UI Text component position and dimensions before initial rendering.
//!             {
//!                 let dim = world.read_resource::<ScreenDimensions>();
//!                 ui_text_size_fn(&mut text_transform, (dim.width(), dim.height()));
//!             }
//!
//!             // If you need to edit the text later, you would want to add the built entity to
//!             // the `world` as a resource. See the `pong` example in the amethyst repository:
//!             //
//!             // <https://github.com/amethyst/amethyst/blob/develop/examples/pong/pong.rs>
//!             // ---
//!             // let text_entity : Entity =
//!             world
//!                 .create_entity()
//!                 .with(text_transform)
//!                 .with(UiText::new(
//!                     font,
//!                     text.to_string(),
//!                     [1., 1., 1., 1.],
//!                     FONT_SIZE,
//!                 ))
//!                 .with(UiResize(Box::new(ui_text_size_fn)))
//!                 .build();
//!         });
//!     }
//!
//!     type FH = FontHandle;
//!     fn read_fonts(world: &mut World) -> (FH, FH, FH, FH) {
//!         // Each of the enum variants corresponds to a font ID.
//!         use application_ui::FontVariant::{Bold, BoldItalic, Italic, Regular};
//!         (
//!             // The `clone()` calls here are necessary to obtain the `FontHandle`, as the
//!             // `read_resource_with_id(..)` function returns a `Fetch` wrapper around the type.
//!             world.read_resource_with_id::<FH>(Regular.into()).clone(),
//!             world.read_resource_with_id::<FH>(Bold.into()).clone(),
//!             world.read_resource_with_id::<FH>(Italic.into()).clone(),
//!             world.read_resource_with_id::<FH>(BoldItalic.into()).clone(),
//!         )
//!     }
//! }
//! ```
//!
//! # Examples
//!
//! See the `01_draw_text` example in this repository, which renders text in regular, bold, italic,
//! and bold italic fonts.

extern crate amethyst;
#[macro_use]
extern crate application;
extern crate ron;
#[macro_use]
extern crate serde;

pub use bundle::ApplicationUiBundle;
pub use font_config::FontConfig;
pub use font_variant::FontVariant;

mod bundle;
mod font_config;
mod font_variant;
