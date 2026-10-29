//! Numeric editing shared by inspectors and settings.

use std::{ffi::CString, os::raw::c_void, ptr};

const DRAG_DIRECTION_THRESHOLD: f32 = 4.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum DragAxis {
    Horizontal,
    Vertical,
}

#[derive(Default)]
pub struct State {
    drag: Option<(dear_imgui_rs::Id, Option<DragAxis>)>,
}

pub trait NumericValue: dear_imgui_rs::internal::DataTypeKind {
    fn offset(&mut self, amount: f32) -> bool;
}

impl NumericValue for f32 {
    fn offset(&mut self, amount: f32) -> bool {
        let next = *self + amount;

        if next == *self {
            return false;
        }

        *self = next;

        true
    }
}

impl NumericValue for i32 {
    fn offset(&mut self, amount: f32) -> bool {
        let amount = amount.round() as i64;

        if amount == 0 {
            return false;
        }

        let next = i64::from(*self)
            .saturating_add(amount)
            .clamp(i64::from(i32::MIN), i64::from(i32::MAX)) as i32;

        if next == *self {
            return false;
        }

        *self = next;

        true
    }
}

impl NumericValue for u32 {
    fn offset(&mut self, amount: f32) -> bool {
        let amount = amount.round() as i64;

        if amount == 0 {
            return false;
        }

        let next = i64::from(*self)
            .saturating_add(amount)
            .clamp(0, i64::from(u32::MAX)) as u32;

        if next == *self {
            return false;
        }

        *self = next;

        true
    }
}

fn numeric_input_arrows<T: NumericValue>(ui: &dear_imgui_rs::Ui, value: &mut T) -> bool {
    if !ui.is_item_active() {
        return false;
    }

    let changed = if ui.is_key_pressed_with_repeat(dear_imgui_rs::Key::UpArrow, true) {
        value.offset(1.0)
    } else if ui.is_key_pressed_with_repeat(dear_imgui_rs::Key::DownArrow, true) {
        value.offset(-1.0)
    } else {
        false
    };

    if changed {
        // InputScalar keeps its own text buffer while editing. Ask it to reload the value
        // written above on the next frame, otherwise it would restore the old text.
        unsafe {
            let state =
                dear_imgui_rs::sys::igGetInputTextState(dear_imgui_rs::sys::igGetActiveID());

            if !state.is_null() {
                dear_imgui_rs::sys::ImGuiInputTextState_ReloadUserBufAndKeepSelection(state);
            }
        }
    }

    changed
}

pub fn drag<T>(
    ui: &dear_imgui_rs::Ui,
    state: &mut State,
    label: &str,
    value: &mut T,
    speed: f32,
    format: &str,
    min: Option<&T>,
    max: Option<&T>,
) -> bool
where
    T: NumericValue,
{
    let label = CString::new(label).expect("Numeric labels must not contain null bytes.");
    let format = dear_imgui_rs::NumericFormat::<T>::new(format).expect("Valid numeric format.");
    let format =
        CString::new(format.as_str()).expect("Numeric formats must not contain null bytes.");
    let flags = dear_imgui_rs::sys::ImGuiSliderFlags_NoRoundToFormat;

    // DataTypeKind fixes the data layout, and both validated strings live through the call.
    let mut changed = unsafe {
        dear_imgui_rs::sys::igDragScalar(
            label.as_ptr(),
            T::KIND as i32,
            value as *mut T as *mut c_void,
            0.0,
            min.map_or(ptr::null(), |value| value as *const T as *const c_void),
            max.map_or(ptr::null(), |value| value as *const T as *const c_void),
            format.as_ptr(),
            flags,
        )
    };

    let text_editing = ui.is_item_active() && ui.io().want_text_input();

    if !ui.is_item_active() {
        if state.drag.is_some_and(|(id, _)| id == ui.item_id()) {
            state.drag = None;
        }
    } else if !text_editing && ui.is_mouse_down(dear_imgui_rs::MouseButton::Left) {
        let item = ui.item_id();

        if ui.is_item_activated() {
            state.drag = Some((item, None));
        }

        if let Some((_, axis)) = state.drag.as_mut().filter(|(id, _)| *id == item) {
            if axis.is_none() {
                let delta = ui.mouse_drag_delta_with_threshold(
                    dear_imgui_rs::MouseButton::Left,
                    DRAG_DIRECTION_THRESHOLD,
                );

                if delta[0].abs().max(delta[1].abs()) >= DRAG_DIRECTION_THRESHOLD {
                    *axis = Some(if delta[1].abs() > delta[0].abs() {
                        DragAxis::Vertical
                    } else {
                        DragAxis::Horizontal
                    });
                }
            }

            let amount = match *axis {
                Some(DragAxis::Vertical) => -ui.io().mouse_delta()[1] * speed,
                Some(DragAxis::Horizontal) => ui.io().mouse_delta()[0] * speed * 10.0,
                None => 0.0,
            };

            changed |= value.offset(amount);
        }
    }

    changed |= numeric_input_arrows(ui, value);

    if ui.is_item_hovered() && !text_editing {
        ui.set_mouse_cursor(Some(dear_imgui_rs::MouseCursor::ResizeAll));
    }

    changed
}

pub fn float_format(value: f32) -> String {
    let text = value.to_string();

    if text.contains(['e', 'E']) {
        return "%.9g".to_owned();
    }

    let decimal_places = text
        .split_once('.')
        .map_or(1, |(_, fraction)| fraction.len());

    if decimal_places > 9 {
        return "%.9g".to_owned();
    }

    format!("%.{}f", decimal_places.max(1))
}

#[cfg(test)]
mod tests {
    use super::{NumericValue, float_format};

    #[test]
    fn float_format_keeps_only_required_decimal_places() {
        assert_eq!(float_format(1.0), "%.1f");
        assert_eq!(float_format(1.2323), "%.4f");
        assert_eq!(float_format(2.4300), "%.2f");
        assert_eq!(float_format(1e-20), "%.9g");
    }

    #[test]
    fn numeric_offsets_round_and_saturate_integer_values() {
        let mut signed = 10i32;

        assert!(signed.offset(-1.4));
        assert_eq!(signed, 9);

        let mut unsigned = 0u32;

        assert!(!unsigned.offset(-1.0));
        assert_eq!(unsigned, 0);
    }
}
