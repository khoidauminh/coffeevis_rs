use softbuffer::{Context, Surface};

use crate::data::log::{error, info};

use qoi;

use winit::{
    application::ApplicationHandler,
    dpi::{PhysicalSize, Size},
    event::WindowEvent,
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    icon::{Icon, RgbaIcon},
    keyboard::{Key, NamedKey},
    window::{Window, WindowAttributes, WindowButtons, WindowId, WindowLevel},
};

use std::{
    cell::LazyCell,
    num::NonZeroU32,
    sync::mpsc::{self, RecvTimeoutError, SyncSender},
    thread::{self, JoinHandle},
    time::Duration,
};

use crate::data::*;

const ICON_FILE: &[u8] = include_bytes!("../../assets/coffeevis_icon_128x128.qoi");

thread_local! {
    static ICON: LazyCell<(u32, u32, Vec<u8>)> = LazyCell::new(|| {
        use qoi::{Decoder, Header};

        let mut icon = Decoder::new(ICON_FILE)
            .map(|i| i.with_channels(qoi::Channels::Rgba))
            .ok()
            .unwrap();

        let &Header { width, height, .. } = icon.header();

        (width, height, icon.decode_to_vec().unwrap())
    });
}

pub struct WindowProps {
    pub width: u16,
    pub height: u16,
}

impl WindowProps {
    pub fn set_size<T: TryInto<u16>>(&mut self, s: (T, T)) {
        self.width = s.0.try_into().ok().unwrap();
        self.height = s.1.try_into().ok().unwrap();
    }
}

struct Renderer {
    pub window: &'static dyn Window,
    pub surface: Surface<&'static dyn Window, &'static dyn Window>,
    pub thread_cursor_id: JoinHandle<()>,
    pub cursor: SyncSender<()>,
}

struct WindowState {
    pub prog: Program,
    pub renderer: Option<Renderer>,
    pub final_buffer_size: PhysicalSize<u32>,
}

impl ApplicationHandler for WindowState {
    fn can_create_surfaces(&mut self, event_loop: &dyn ActiveEventLoop) {
        // Prevents re-initialization.
        assert!(self.renderer.is_none());

        let scale = self.prog.scale() as u32;
        let win_size = PhysicalSize::<u32>::new(
            self.prog.pix.width() as u32 * scale,
            self.prog.pix.height() as u32 * scale,
        );

        let de = std::env::var("XDG_CURRENT_DESKTOP").unwrap_or_default();

        let icon = ICON.with(|icon| {
            RgbaIcon::new(icon.2.to_vec(), icon.0, icon.1)
                .inspect_err(|_| error!("Failed to create window icon."))
                .ok()
                .map(Icon::from)
        });

        let wm_attr_wayland = winit::platform::wayland::WindowAttributesWayland::default()
            .with_name("Coffeevis", "cvis");

        let window_attributes = WindowAttributes::default()
            .with_platform_attributes(Box::new(wm_attr_wayland))
            .with_title("cvis")
            .with_surface_size(win_size)
            .with_window_level(WindowLevel::AlwaysOnTop)
            .with_transparent(false)
            .with_resizable(self.prog.is_resizable())
            .with_visible(false)
            .with_enabled_buttons(WindowButtons::CLOSE)
            .with_window_icon(icon);

        let window = &*Box::leak(
            event_loop
                .create_window(window_attributes)
                .expect("Failed to create window!"),
        );

        let size = window.surface_size();
        self.final_buffer_size = size;

        let surface = {
            let mut s = Surface::new(&Context::new(window).unwrap(), window).unwrap();

            s.resize(
                NonZeroU32::new(win_size.width).unwrap(),
                NonZeroU32::new(win_size.height).unwrap(),
            )
            .unwrap();

            s
        };

        if !de.is_empty() {
            info!("Running in the {} desktop (XDG_CURRENT_DESKTOP).", de);
        }

        let limit_size = Size::Physical(size);

        crate::audio::get_buf().init_realtime_wakeup(window);

        // On XFCE this is needed to lock the size of the window.
        if !self.prog.is_resizable() {
            window.set_min_surface_size(Some(limit_size));
            window.set_max_surface_size(Some(limit_size));
        } else {
            window.set_max_surface_size(Some(Size::Physical(PhysicalSize::new(
                MAX_WIDTH as u32,
                MAX_HEIGHT as u32,
            ))));
        }

        window.set_visible(true);
        window.set_window_level(WindowLevel::AlwaysOnTop);
        window.request_redraw();

        let (cursor_sender, cursor_receiver) = mpsc::sync_channel(1);

        let thread_cursor_id = thread::Builder::new()
            .name("coffeevis cursor control".into())
            .spawn(move || {
                let mut waitfor = Duration::from_secs(1);

                loop {
                    match cursor_receiver.recv_timeout(waitfor) {
                        Ok(()) => {
                            waitfor = Duration::from_secs(1);
                        }
                        Err(RecvTimeoutError::Timeout) => {
                            waitfor = Duration::from_secs(60);
                            window.set_cursor_visible(false);
                        }
                        Err(_) => break,
                    }
                }
            })
            .unwrap();

        self.renderer = Some(Renderer {
            window,
            surface,
            thread_cursor_id,
            cursor: cursor_sender,
        })
    }

    fn window_event(&mut self, event_loop: &dyn ActiveEventLoop, _: WindowId, event: WindowEvent) {
        let Some(Renderer {
            window,
            surface,
            cursor,
            ..
        }) = self.renderer.as_mut()
        else {
            return;
        };

        if self.prog.nosleep() {
            window.request_redraw();
        }

        match event {
            WindowEvent::RedrawRequested => {
                let Ok(mut buffer) = surface.buffer_mut() else {
                    return;
                };

                let mut buf = crate::audio::get_buf();
                if buf.silent() < STOP_RENDERING {
                    window.request_redraw();
                }

                self.prog.render(&mut buf);

                self.prog.pix.scale_to(
                    self.prog.scale() as usize,
                    &mut buffer,
                    Some(self.final_buffer_size.width as usize),
                    self.prog.get_win_render_effect(),
                );

                window.pre_present_notify();

                if let Err(e) = buffer.present() {
                    error!(
                        "Coffeevis is failing to present buffers to the window: {}!",
                        e
                    );
                }
            }

            WindowEvent::CloseRequested => self.call_exit(event_loop),

            WindowEvent::PointerMoved { .. } => {
                if let Some(err) = window.drag_window().err() {
                    error!("Error dragging window: {}", err);
                }

                window.set_cursor_visible(true);
                window.request_redraw();

                let _ = cursor.try_send(());
            }

            WindowEvent::Focused(_) => {
                window.request_redraw();
            }

            WindowEvent::SurfaceResized(PhysicalSize { width, height }) => {
                let scale = self.prog.scale() as u16;

                let w = u16::min(MAX_WIDTH, width as u16);
                let h = u16::min(MAX_HEIGHT, height as u16);

                self.prog.update_size((w / scale, h / scale));

                let (w, h) = (w as u32, h as u32);

                self.final_buffer_size.width = w;
                self.final_buffer_size.height = h;

                surface
                    .resize(
                        NonZeroU32::new(w).expect("Surface width is zero"),
                        NonZeroU32::new(h).expect("Surface height is zero"),
                    )
                    .expect("Failed to resize surface buffer");

                self.prog.pix.clear_out_buffer();
            }

            WindowEvent::KeyboardInput { event, .. } if !event.repeat => {
                let pressed = event.state.is_pressed();
                let k = event.key_without_modifiers.as_ref();

                window.request_redraw();

                if pressed {
                    match k {
                        Key::Character("q") => self.call_exit(event_loop),
                        Key::Character("n") => self.prog.change_visualizer(true),
                        Key::Character("b") => self.prog.change_visualizer(false),
                        Key::Character("\\") => self.prog.toggle_auto_switch(),
                        Key::Character("/") => self.prog.reset_parameters(),
                        _ => {}
                    }
                }

                match k {
                    Key::Character("z") => self.prog.key.z = pressed,
                    Key::Character("x") => self.prog.key.x = pressed,
                    Key::Character("c") => self.prog.key.c = pressed,

                    Key::Named(NamedKey::ArrowLeft) => self.prog.key.left = pressed,
                    Key::Named(NamedKey::ArrowDown) => self.prog.key.down = pressed,
                    Key::Named(NamedKey::ArrowUp) => self.prog.key.up = pressed,
                    Key::Named(NamedKey::ArrowRight) => self.prog.key.right = pressed,

                    _ => {}
                }
            }

            _ => {}
        }
    }
}

impl WindowState {
    fn call_exit(&mut self, event_loop: &dyn ActiveEventLoop) {
        if let Some(t) = self.renderer.take() {
            crate::audio::get_buf().close_notifier();
            drop(t.cursor);
            t.thread_cursor_id.join().unwrap();
        }

        event_loop.exit();
    }
}

pub fn winit_main(prog: Program) {
    let event_loop = EventLoop::new().unwrap();

    prog.print_startup_info();

    event_loop.set_control_flow(ControlFlow::Wait);
    event_loop
        .run_app(WindowState {
            prog,
            renderer: None,
            final_buffer_size: PhysicalSize::<u32>::new(0, 0),
        })
        .unwrap();
}
