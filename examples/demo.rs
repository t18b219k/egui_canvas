use instant::Instant;

use egui::{Color32, FontData, FullOutput};
use egui_winit_platform::{Platform, PlatformDescriptor};
use epi::*;
use winit::dpi::LogicalPosition;
use winit::event::Event::*;
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::Window;

struct RepaintSignalMock;

impl epi::backend::RepaintSignal for RepaintSignalMock {
    fn request_repaint(&self) {}
}

/// A simple egui + wgpu + winit based example.
fn run(event_loop: EventLoop<()>, window: Window) {
    use winit::platform::web::WindowExtWebSys;
    let canvas = window.canvas();

    let mut renderer = egui_canvas::Renderer::new_with_canvas(&canvas).unwrap();

    let size = window.inner_size();

    let repaint_signal = std::sync::Arc::new(RepaintSignalMock);
    let mut fonts = egui::FontDefinitions::default();
    fonts.font_data.insert(
        "Noto".to_owned(),
        FontData::from_static(include_bytes!("./NotoSansJP-Regular.otf")),
    );
    fonts.families.iter_mut().for_each(|(_, fonts)| {
        fonts.insert(0, "Noto".to_owned());
    });

    // We use the egui_winit_platform crate as the platform.
    let mut platform = Platform::new(PlatformDescriptor {
        physical_width: size.width as u32,
        physical_height: size.height as u32,
        scale_factor: window.scale_factor(),
        font_definitions: fonts,
        style: Default::default(),
    });

    // Display the demo application that ships with egui.
    let mut demo_app = crate::keyboard_debugger::KeyboardDebugger::new();

    let start_time = Instant::now();
    let mut previous_frame_time = None;
    event_loop.run(move |event, _, control_flow| {
        // Pass the winit events to the platform integration.
        platform.handle_event(&event);
        match event {
            RedrawRequested(..) => {
                platform.update_time(start_time.elapsed().as_secs_f64());

                // Begin to draw the UI frame.
                let egui_start = Instant::now();
                platform.begin_frame();
                let app_output = epi::backend::AppOutput::default();

                let mut frame = epi::Frame::new(epi::backend::FrameData {
                    info: epi::IntegrationInfo {
                        name: "egui_example",
                        web_info: None,
                        cpu_usage: previous_frame_time,
                        native_pixels_per_point: Some(window.scale_factor() as _),
                        prefer_dark_mode: None,
                    },
                    output: app_output,
                    repaint_signal: repaint_signal.clone(),
                });

                // Draw the demo application.
                demo_app.update(&platform.context(), &mut frame);

                // End the UI frame. We could now handle the output and draw the UI with the backend.
                let FullOutput {
                    platform_output,
                    needs_repaint,
                    textures_delta,
                    shapes,
                } = platform.end_frame(Some(&window));
                if let Some(pos) = platform_output.text_cursor_pos {
                    window.set_ime_position(LogicalPosition::new(pos.x, pos.y));
                }
                let frame_time = (Instant::now() - egui_start).as_secs_f64() as f32;
                previous_frame_time = Some(frame_time);

                renderer.clear(&Color32::BLACK);
                renderer.paint_and_update_texture(&shapes, textures_delta);
                if needs_repaint {
                    window.request_redraw();
                }
                log::info!("rendered");
            }
            MainEventsCleared => {
                log::info!("redraw request");
                window.request_redraw();
            }
            WindowEvent { event, .. } => match event {
                winit::event::WindowEvent::CloseRequested => {
                    *control_flow = ControlFlow::Exit;
                }
                event => match event {
                    winit::event::WindowEvent::ReceivedCharacter(_)
                    | winit::event::WindowEvent::KeyboardInput { .. }
                    | winit::event::WindowEvent::IME(_) => {
                        demo_app.feed(&event);
                    }
                    _ => (),
                },
            },
            _ => (),
        }
    });
}

pub fn main() {
    let event_loop = EventLoop::new();

    use winit::platform::web::WindowExtWebSys;
    let window = winit::window::WindowBuilder::new()
        .build(&event_loop)
        .unwrap();
    let web_window = web_sys::window().unwrap();
    let document = web_window.document().unwrap();
    let body = document.body().unwrap();

    body.append_child(&window.canvas()).unwrap();

    run(event_loop, window);
}

#[cfg(target_arch = "wasm32")]
mod wasm {
    use wasm_bindgen::prelude::wasm_bindgen;

    #[wasm_bindgen]
    pub fn start() {
        std::panic::set_hook(Box::new(console_error_panic_hook::hook));
        use log::Level;
        console_log::init_with_level(Level::Trace).expect("failed to init logger");

        crate::main()
    }
}
mod keyboard_debugger {
    use egui::Context;
    use epi::Frame;

    pub struct KeyboardDebugger {
        text_buffer: String,
        event_buffer: Vec<String>,
    }

    impl KeyboardDebugger {
        pub fn new() -> KeyboardDebugger {
            Self {
                text_buffer: String::new(),
                event_buffer: vec![],
            }
        }
        pub fn feed(&mut self, event: &winit::event::WindowEvent) {
            self.event_buffer.push(format!("{:?}", event));
        }
        pub fn clear(&mut self) {
            self.event_buffer.clear()
        }
    }
    impl epi::App for KeyboardDebugger {
        fn update(&mut self, ctx: &Context, frame: &Frame) {
            egui::Window::new("Keyboard debugger").show(ctx, |ui| {
                ui.vertical(|ui| {
                    ui.horizontal(|ui| {
                        ui.label("please input here");
                        ui.text_edit_singleline(&mut self.text_buffer);
                        if ui.button("clear logs").clicked() {
                            self.clear()
                        }
                    });
                    let scroll = egui::containers::ScrollArea::both();
                    scroll.stick_to_bottom().show(ui, |ui| {
                        ui.vertical(|ui| {
                            self.event_buffer.iter().for_each(|log| {
                                ui.label(log);
                            });
                        })
                    });
                });
            });
            frame.request_repaint()
        }

        fn name(&self) -> &str {
            "Keyboard debugger"
        }
    }
}
