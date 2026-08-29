//! Settings modal shell.
//!
//! Settings used to *replace* the workspace view: the left rail became the
//! settings nav and the pane grid became the section panel. Two surfaces
//! swapping at once made it hard to see what had actually changed, and the way
//! out was a single "Back to the app" row buried at the top of the rail.
//!
//! They are now a centered modal layered over the untouched workspace, built
//! from the same recipe as every other overlay in the app (`theme_picker`,
//! `custom_buttons_modal`): a dimming backdrop, an `occlude()`d card that owns
//! focus and the key handler, and the whole thing `deferred()` so it paints
//! above the pane grid. The card reuses [`PaneFlowApp::render_settings_nav`]
//! and [`PaneFlowApp::render_settings_content_panel`] verbatim - only the
//! frame changed, not the settings themselves.

use gpui::{
    AnyElement, ClickEvent, Context, FontWeight, InteractiveElement, IntoElement, MouseButton,
    ParentElement, StatefulInteractiveElement, Styled, Window, deferred, div, hsla, px, relative,
    svg,
};

use crate::PaneFlowApp;
use crate::ui_primitives::{AnimatedHoverExt, lerp_color};

/// Preferred card width, capped against the window so a narrow window shrinks
/// the modal instead of overflowing it.
const MODAL_WIDTH: f32 = 1000.;
const MODAL_MAX_WIDTH_FRACTION: f32 = 0.92;

/// Height is a fraction of the window rather than a fixed size: settings pages
/// are long lists, so the card should use the screen it is given. A fixed
/// height left the modal short and floating on a tall display. The cap keeps
/// it from turning into an unreadable full-height column on a large monitor.
const MODAL_HEIGHT_FRACTION: f32 = 0.86;
const MODAL_MAX_HEIGHT: f32 = 1000.;

impl PaneFlowApp {
    /// The settings modal: backdrop + centered card (header, nav rail, section
    /// panel). Mounted from `main.rs` whenever `settings_section` is set.
    pub(crate) fn render_settings_modal(
        &self,
        window: &Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let ui = crate::theme::ui_colors();

        // Header: the title the nav rail used to spend a row on, plus the
        // close affordance. A modal header is where users look for the way
        // out, which the old rail-embedded "Back to the app" row was not.
        let header = div()
            .flex()
            .flex_row()
            .items_center()
            .justify_between()
            .gap(px(12.))
            .flex_none()
            .px(px(16.))
            .pt(px(14.))
            .pb(px(12.))
            .border_b_1()
            .border_color(ui.border)
            .child(
                div()
                    .text_size(px(13.))
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(ui.text)
                    .child("Settings"),
            )
            .child(
                div()
                    .id("settings-header-close")
                    .flex()
                    .items_center()
                    .justify_center()
                    .w(px(22.))
                    .h(px(22.))
                    .rounded(px(4.))
                    .animated_hover(move |style, delta| {
                        style.bg(lerp_color(ui.subtle.opacity(0.0), ui.subtle, delta));
                    })
                    .on_click(cx.listener(|this, _: &ClickEvent, _window, cx| {
                        this.close_settings(cx);
                        cx.stop_propagation();
                        cx.notify();
                    }))
                    .child(
                        svg()
                            .size(px(11.))
                            .flex_none()
                            .path("icons/close.svg")
                            .text_color(ui.muted),
                    ),
            );

        // Body: the two existing surfaces, side by side inside the card.
        let body = div()
            .flex()
            .flex_row()
            .flex_1()
            .min_h_0()
            .overflow_hidden()
            .child(self.render_settings_nav(window, cx))
            .child(self.render_settings_content_panel(cx));

        let card = div()
            .id("settings-modal")
            .occlude()
            .track_focus(&self.settings_focus)
            .on_key_down(cx.listener(Self::handle_settings_key_down))
            // Clicking the dimmed backdrop leaves settings; clicks landing on
            // the card must not travel out to it.
            .on_mouse_down_out(cx.listener(|this, _, _window, cx| {
                this.close_settings(cx);
                cx.notify();
            }))
            .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
            .on_mouse_down(MouseButton::Right, |_, _, cx| cx.stop_propagation())
            .w(px(MODAL_WIDTH))
            .max_w(relative(MODAL_MAX_WIDTH_FRACTION))
            .h(relative(MODAL_HEIGHT_FRACTION))
            .max_h(px(MODAL_MAX_HEIGHT))
            .flex()
            .flex_col()
            .bg(ui.overlay)
            .border_1()
            .border_color(ui.border)
            .rounded(px(10.))
            .shadow_lg()
            .overflow_hidden()
            .child(header)
            .child(body);

        let backdrop = div()
            .id("settings-backdrop")
            .absolute()
            .top_0()
            .left_0()
            .size_full()
            .flex()
            .items_center()
            .justify_center()
            .bg(hsla(0., 0., 0., 0.45))
            .child(card);

        deferred(backdrop).with_priority(8).into_any_element()
    }
}
