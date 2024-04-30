// This will all be rewritten soon, looking toward per-widget cursor control
// Rewrite should address issue #93 too

use crate::*;
use bevy::{input::mouse::MouseMotion, prelude::*, window::PrimaryWindow};

/// System set for mouse cursor systems. Runs in [`Update`]
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct CursorSet;

/// Config enum for mouse cursor events.
#[derive(Default, Clone)]
pub enum CursorConfig {
    /// Emit [`TextHoverIn`] and [`TextHoverOut`] events and change mouse cursor on hover
    #[default]
    Default,
    /// Emit [`TextHoverIn`] and [`TextHoverOut`] events, but do not change the cursor
    Events,
    /// Ignore mouse events
    None,
}

pub(crate) struct CursorPlugin {
    pub change_cursor: CursorConfig,
}

impl Plugin for CursorPlugin {
    fn build(&self, app: &mut App) {
        match self.change_cursor {
            CursorConfig::Default => {
                app.add_systems(Update, (hover_sprites, hover_ui, change_cursor))
                    .add_event::<TextHoverIn>()
                    .add_event::<TextHoverOut>();
            }
            CursorConfig::Events => {
                app.add_systems(Update, (hover_sprites, hover_ui))
                    .add_event::<TextHoverIn>()
                    .add_event::<TextHoverOut>();
            }
            CursorConfig::None => {}
        }
    }
}

/// For use with custom cursor control; Event is emitted when cursor enters a text widget
#[derive(Event)]
pub struct TextHoverIn;

/// For use with custom cursor control; Event is emitted when cursor leaves a text widget
#[derive(Event)]
pub struct TextHoverOut;

/// Switches mouse cursor icon when hover events are received
pub fn change_cursor(
    evr_hover_in: EventReader<TextHoverIn>,
    evr_hover_out: EventReader<TextHoverOut>,
    evr_text_changed: EventReader<CosmicTextChanged>,
    evr_mouse_motion: EventReader<MouseMotion>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    mut windows: Query<&mut Window, With<PrimaryWindow>>,
) {
    if windows.iter().len() == 0 {
        return;
    }
    let mut window = windows.single_mut();
    if !evr_hover_in.is_empty() {
        window.cursor.icon = CursorIcon::Text;
    }
    if !evr_hover_out.is_empty() {
        window.cursor.icon = CursorIcon::Default;
    }
    if !evr_text_changed.is_empty() {
        window.cursor.visible = false;
    }
    if mouse_buttons.get_just_pressed().len() != 0 || !evr_mouse_motion.is_empty() {
        window.cursor.visible = true;
    }
}

#[cfg(feature = "multicam")]
type CameraQuery<'a, 'b, 'c, 'd> =
    Query<'a, 'b, (&'c Camera, &'d GlobalTransform), With<CosmicPrimaryCamera>>;

#[cfg(not(feature = "multicam"))]
type CameraQuery<'a, 'b, 'c, 'd> = Query<'a, 'b, (&'c Camera, &'d GlobalTransform)>;

/// Sprite widget mouse cursor hover detection system. Sends [`TextHoverIn`] and [`TextHoverOut`]
/// events.
pub fn hover_sprites(
    windows: Query<&Window, With<PrimaryWindow>>,
    mut cosmic_edit_query: Query<(&mut Sprite, &Visibility, &GlobalTransform), With<CosmicBuffer>>,
    camera_q: CameraQuery,
    mut hovered: Local<bool>,
    mut last_hovered: Local<bool>,
    mut evw_hover_in: EventWriter<TextHoverIn>,
    mut evw_hover_out: EventWriter<TextHoverOut>,
) {
    *hovered = false;
    if windows.iter().len() == 0 {
        return;
    }
    let window = windows.single();
    let (camera, camera_transform) = camera_q.single();
    for (sprite, visibility, node_transform) in &mut cosmic_edit_query.iter_mut() {
        if visibility == Visibility::Hidden {
            continue;
        }

        let size = sprite.custom_size.unwrap_or(Vec2::ONE);
        let x_min = node_transform.affine().translation.x - size.x / 2.;
        let y_min = node_transform.affine().translation.y - size.y / 2.;
        let x_max = node_transform.affine().translation.x + size.x / 2.;
        let y_max = node_transform.affine().translation.y + size.y / 2.;
        if let Some(pos) = window.cursor_position() {
            if let Some(pos) = camera.viewport_to_world_2d(camera_transform, pos) {
                if x_min < pos.x && pos.x < x_max && y_min < pos.y && pos.y < y_max {
                    *hovered = true;
                }
            }
        }
    }

    if *last_hovered != *hovered {
        if *hovered {
            evw_hover_in.send(TextHoverIn);
        } else {
            evw_hover_out.send(TextHoverOut);
        }
    }

    *last_hovered = *hovered;
}

/// UI widget mouse cursor hover detection system. Sends [`TextHoverIn`] and [`TextHoverOut`]
/// events.
pub fn hover_ui(
    mut interaction_query: Query<&Interaction, (Changed<Interaction>, With<CosmicSource>)>,
    mut evw_hover_in: EventWriter<TextHoverIn>,
    mut evw_hover_out: EventWriter<TextHoverOut>,
) {
    for interaction in interaction_query.iter_mut() {
        match interaction {
            Interaction::None => {
                evw_hover_out.send(TextHoverOut);
            }
            Interaction::Hovered => {
                evw_hover_in.send(TextHoverIn);
            }
            _ => {}
        }
    }
}
