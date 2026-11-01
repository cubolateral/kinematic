//! Fades transient window geometry without extending its interactive lifetime.

use dear_imgui_rs::{Ui, WindowFlags, sys};

use super::OVERLAY_DURATION;

#[derive(Default)]
pub struct Overlays {
    context: usize,
    frame: usize,
    windows: Vec<Overlay>,
}

struct Overlay {
    id: u32,
    viewport: u32,
    seen: bool,
    opacity: f32,
    start: f32,
    elapsed: f32,
    visible: bool,
    meshes: Vec<Mesh>,
}

struct Mesh {
    vertices: Vec<sys::ImDrawVert>,
    indices: Vec<sys::ImDrawIdx>,
    commands: Vec<Command>,
}

struct Command {
    clip: sys::ImVec4,
    texture: Texture,
    vertices: usize,
    indices: std::ops::Range<usize>,
}

enum Texture {
    Managed(i32),
    External(sys::ImTextureID),
}

impl Overlay {
    fn advance(&mut self, visible: bool, dt: f32) {
        if self.visible != visible {
            self.start = self.opacity;
            self.elapsed = 0.0;
            self.visible = visible;
        }

        self.elapsed = (self.elapsed + dt).min(OVERLAY_DURATION);

        let progress = self.elapsed / OVERLAY_DURATION;
        let eased = if visible {
            1.0 - (1.0 - progress).powi(3)
        } else {
            progress.powi(3)
        };

        self.opacity = self.start + (f32::from(visible) - self.start) * eased;
    }
}

impl Overlays {
    /// Call after all UI windows have ended, before rendering the ImGui frame.
    pub fn draw(&mut self, ui: &Ui) {
        // Native access is needed to include child windows and CTE's packed vertex colors.
        // Only owned geometry and texture IDs survive the frame; no native pointers are retained.
        ui.with_bound_context(|| unsafe {
            let context = sys::igGetCurrentContext();
            let frame = ui.frame_count();

            if self.context != context as usize || frame < self.frame {
                self.windows.clear();

                self.context = context as usize;
            } else if frame == self.frame {
                return;
            }

            self.frame = frame;

            for overlay in &mut self.windows {
                overlay.seen = false;
            }

            let dt = ui.io().delta_time();
            let mut windows =
                native_slice((*context).Windows.Data, (*context).Windows.Size).to_vec();

            windows.sort_by_key(|&window| (*window).BeginOrderWithinContext);

            for window in windows {
                let root = (*window).RootWindow;

                if !(*window).Active
                    || (*window).Hidden
                    || root.is_null()
                    || !transient((*root).Flags)
                {
                    continue;
                }

                let draw = (*window).DrawList;

                if draw.is_null() || (*draw).VtxBuffer.Size == 0 {
                    continue;
                }

                let popup_closed = (*root).Flags & sys::ImGuiWindowFlags_Popup as i32 != 0
                    && !native_slice(
                        (*context).OpenPopupStack.Data,
                        (*context).OpenPopupStack.Size,
                    )
                    .iter()
                    .any(|popup| popup.PopupId == (*root).PopupId);

                if popup_closed || (*root).SkipItems {
                    // Closing windows can submit a background after their contents have disappeared.
                    for vertex in native_slice_mut((*draw).VtxBuffer.Data, (*draw).VtxBuffer.Size) {
                        vertex.col = fade(vertex.col, 0.0);
                    }

                    continue;
                }

                let id = (*root).ID;
                let index = self
                    .windows
                    .iter()
                    .position(|overlay| overlay.id == id)
                    .unwrap_or_else(|| {
                        self.windows.push(Overlay {
                            id,
                            viewport: (*root).ViewportId,
                            seen: false,
                            opacity: 0.0,
                            start: 0.0,
                            elapsed: 0.0,
                            visible: true,
                            meshes: Vec::new(),
                        });

                        self.windows.len() - 1
                    });
                let overlay = &mut self.windows[index];

                if !overlay.seen {
                    overlay.meshes.clear();

                    overlay.viewport = (*root).ViewportId;

                    overlay.advance(true, dt);

                    overlay.seen = true;
                }

                let mesh = Mesh::capture(draw);
                let vertices = native_slice_mut((*draw).VtxBuffer.Data, (*draw).VtxBuffer.Size);

                for vertex in vertices {
                    vertex.col = fade(vertex.col, overlay.opacity);
                }

                overlay.meshes.push(mesh);
            }

            for overlay in &mut self.windows {
                if overlay.seen {
                    continue;
                }

                overlay.advance(false, dt);

                let viewport = sys::igFindViewportByID(overlay.viewport);

                if viewport.is_null() || overlay.opacity == 0.0 {
                    continue;
                }

                let draw = sys::igGetForegroundDrawList_ViewportPtr(viewport);

                for mesh in &overlay.meshes {
                    mesh.replay(context, draw, overlay.opacity);
                }
            }

            self.windows
                .retain(|overlay| overlay.seen || overlay.opacity > 0.0);
        });
    }
}

fn transient(flags: i32) -> bool {
    let popup = flags & (sys::ImGuiWindowFlags_Popup | sys::ImGuiWindowFlags_Tooltip) as i32 != 0;
    let flags = WindowFlags::from_bits_truncate(flags);

    popup
        || (flags.contains(WindowFlags::NO_DOCKING | WindowFlags::NO_SAVED_SETTINGS)
            && (!flags.contains(WindowFlags::NO_TITLE_BAR)
                || flags.contains(WindowFlags::NO_FOCUS_ON_APPEARING)))
}

fn fade(color: u32, opacity: f32) -> u32 {
    (color & 0x00ff_ffff) | ((((color >> 24) as f32 * opacity).round() as u32) << 24)
}

impl Mesh {
    unsafe fn capture(draw: *mut sys::ImDrawList) -> Self {
        unsafe {
            Self {
                vertices: native_slice((*draw).VtxBuffer.Data, (*draw).VtxBuffer.Size).to_vec(),
                indices: native_slice((*draw).IdxBuffer.Data, (*draw).IdxBuffer.Size).to_vec(),
                commands: native_slice((*draw).CmdBuffer.Data, (*draw).CmdBuffer.Size)
                    .iter()
                    .filter(|command| command.UserCallback.is_none() && command.ElemCount > 0)
                    .map(|command| Command {
                        clip: command.ClipRect,
                        texture: if command.TexRef._TexData.is_null() {
                            Texture::External(command.TexRef._TexID)
                        } else {
                            Texture::Managed((*command.TexRef._TexData).UniqueID)
                        },
                        vertices: command.VtxOffset as usize,
                        indices: command.IdxOffset as usize
                            ..(command.IdxOffset + command.ElemCount) as usize,
                    })
                    .collect(),
            }
        }
    }

    unsafe fn replay(
        &self,
        context: *mut sys::ImGuiContext,
        draw: *mut sys::ImDrawList,
        opacity: f32,
    ) {
        unsafe {
            for command in &self.commands {
                let texture = match command.texture {
                    Texture::External(id) => sys::ImTextureRef {
                        _TexID: id,
                        _TexData: std::ptr::null_mut(),
                    },
                    Texture::Managed(id) => {
                        let atlas = (*context).IO.Fonts;
                        let textures = native_slice((*atlas).TexList.Data, (*atlas).TexList.Size);
                        let Some(&texture) = textures.iter().find(|&&texture| {
                            (*texture).UniqueID == id
                                && !(*texture).WantDestroyNextFrame
                                && (*texture).Status != sys::ImTextureStatus_WantDestroy as i32
                                && (*texture).Status != sys::ImTextureStatus_Destroyed as i32
                        }) else {
                            continue;
                        };

                        sys::ImTextureRef {
                            _TexID: 0,
                            _TexData: texture,
                        }
                    }
                };

                sys::ImDrawList_PushClipRect(
                    draw,
                    sys::ImVec2 {
                        x: command.clip.x,
                        y: command.clip.y,
                    },
                    sys::ImVec2 {
                        x: command.clip.z,
                        y: command.clip.w,
                    },
                    false,
                );
                sys::ImDrawList_PushTexture(draw, texture);
                // Chunking keeps each reservation within ImGui's 16-bit index range.
                for indices in self.indices[command.indices.clone()].chunks(60_000) {
                    sys::ImDrawList_PrimReserve(draw, indices.len() as i32, indices.len() as i32);

                    for &index in indices {
                        let vertex = &self.vertices[command.vertices + index as usize];

                        sys::ImDrawList_PrimVtx(
                            draw,
                            vertex.pos,
                            vertex.uv,
                            fade(vertex.col, opacity),
                        );
                    }
                }

                sys::ImDrawList_PopTexture(draw);
                sys::ImDrawList_PopClipRect(draw);
            }
        }
    }
}

unsafe fn native_slice<'a, T>(data: *const T, size: i32) -> &'a [T] {
    if size == 0 {
        &[]
    } else {
        unsafe { std::slice::from_raw_parts(data, size as usize) }
    }
}

unsafe fn native_slice_mut<'a, T>(data: *mut T, size: i32) -> &'a mut [T] {
    if size == 0 {
        &mut []
    } else {
        unsafe { std::slice::from_raw_parts_mut(data, size as usize) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn closing_a_modal_keeps_its_last_complete_contents_during_the_fade() {
        let _lock = crate::TEST_LOCK.lock().unwrap();
        let mut context = dear_imgui_rs::Context::create();

        context.set_ini_filename(None::<&str>).unwrap();
        context.io_mut().set_display_size([800.0, 600.0]);
        context.io_mut().set_delta_time(OVERLAY_DURATION / 4.0);
        context
            .font_atlas()
            .try_claim_legacy_renderer()
            .unwrap()
            .build();

        let mut overlays = Overlays::default();
        let mut complete = 0;

        for frame in 0..10 {
            let ui = context.frame();

            if frame == 0 {
                ui.open_popup("Closing modal");
            }

            let mut open = frame < 5;

            if let Some(_popup) =
                crate::windows::sized_modal(ui, "Closing modal", &mut open, [480.0, 240.0])
            {
                ui.text("This text must remain visible throughout the closing animation.");
                ui.child_window("Contents").build(ui, || {
                    ui.text("Child contents must remain visible too.");
                });
            }

            overlays.draw(ui);

            if frame == 4 {
                complete = overlays.windows[0]
                    .meshes
                    .iter()
                    .map(|mesh| mesh.indices.len())
                    .sum();

                assert!(complete > 100);
            }

            if (5..8).contains(&frame) {
                assert!(!ui.is_popup_open("Closing modal"));

                let overlay = &overlays.windows[0];

                assert!(!overlay.visible);
                assert!(overlay.opacity > 0.0);

                let retained: usize = overlay.meshes.iter().map(|mesh| mesh.indices.len()).sum();

                assert_eq!(retained, complete);

                ui.with_bound_context(|| unsafe {
                    let draw = sys::igGetForegroundDrawList_ViewportPtr(sys::igGetMainViewport());

                    assert_eq!((*draw).IdxBuffer.Size as usize, complete);
                });
            }

            drop(context.render_legacy());
        }
    }

    #[test]
    fn tooltips_popups_and_documentation_fade_with_children_and_release_input_on_close() {
        let _lock = crate::TEST_LOCK.lock().unwrap();

        for modal in [false, true] {
            let mut context = dear_imgui_rs::Context::create();

            context.set_ini_filename(None::<&str>).unwrap();
            context.io_mut().set_display_size([800.0, 600.0]);
            context.io_mut().set_delta_time(OVERLAY_DURATION / 4.0);
            context
                .font_atlas()
                .try_claim_legacy_renderer()
                .unwrap()
                .build();

            let mut overlays = Overlays::default();

            for frame in 0..12 {
                let ui = context.frame();
                let mut child_id = 0;
                let mut child_color = 0;

                ui.window("Editor").build(|| {
                    if frame == 0 {
                        ui.open_popup("Popup");
                    }

                    if frame < 6 {
                        ui.tooltip(|| ui.text("Tooltip content."));

                        let content = || {
                            ui.text("Popup content.");

                            if frame == 5 {
                                ui.close_current_popup();
                            }
                        };

                        if modal {
                            let mut open = true;

                            if let Some(_popup) = crate::windows::modal(ui, "Popup", &mut open) {
                                content();
                            }
                        } else if let Some(_popup) = ui.begin_popup("Popup") {
                            content();
                        }

                        crate::windows::anchored(
                            ui,
                            "Documentation",
                            [300.0, 100.0],
                            Some([300.0, 200.0]),
                            WindowFlags::empty(),
                            || {
                                ui.child_window("Code").build(ui, || {
                                    ui.text("Documentation content.");

                                    child_id = ui.with_bound_context(|| unsafe {
                                        (*sys::igGetCurrentWindow()).ID
                                    });

                                    child_color = 0xff44_88cc;

                                    ui.get_window_draw_list()
                                        .add_rect([310.0, 110.0], [320.0, 120.0], child_color)
                                        .filled(true)
                                        .build();
                                });
                            },
                        );
                    }

                    if frame == 5 {
                        assert!(!ui.is_popup_open("Popup"));
                    }
                });

                overlays.draw(ui);

                if frame == 2 {
                    assert!(overlays.windows.len() >= 3);
                    assert!(
                        overlays
                            .windows
                            .iter()
                            .all(|overlay| overlay.opacity > 0.0 && overlay.opacity < 1.0)
                    );

                    ui.with_bound_context(|| unsafe {
                        let window = sys::igFindWindowByID(child_id);
                        let draw = (*window).DrawList;
                        let vertices = native_slice((*draw).VtxBuffer.Data, (*draw).VtxBuffer.Size);
                        let color = vertices.last().unwrap().col;

                        assert_eq!(color & 0x00ff_ffff, child_color & 0x00ff_ffff);
                        assert!(color >> 24 > 0 && color >> 24 < 255);
                    });
                }

                if frame == 6 {
                    assert!(
                        overlays
                            .windows
                            .iter()
                            .all(|overlay| !overlay.visible && overlay.opacity > 0.0)
                    );

                    ui.with_bound_context(|| unsafe {
                        let draw =
                            sys::igGetForegroundDrawList_ViewportPtr(sys::igGetMainViewport());

                        assert!((*draw).VtxBuffer.Size > 0);
                    });
                }

                if frame >= 10 {
                    assert!(overlays.windows.is_empty());
                }

                let rendered = context.render_legacy();

                for list in rendered.draw_data().draw_lists() {
                    for command in list.commands() {
                        if let dear_imgui_rs::render::DrawCmd::Elements { count, cmd_params } =
                            command
                        {
                            for &index in &list.idx_buffer()
                                [cmd_params.idx_offset..cmd_params.idx_offset + count]
                            {
                                assert!(
                                    (index as usize) + cmd_params.vtx_offset
                                        < list.vtx_buffer().len()
                                );
                            }
                        }
                    }
                }
            }
        }
    }
}
