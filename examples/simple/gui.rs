mod controls;

use std::cell::RefCell;
use std::os::raw::c_void;

use cocoa::appkit::{NSBackingStoreType, NSView, NSWindow, NSWindowStyleMask};
use cocoa::base::{id, nil};
use cocoa::foundation::{NSPoint, NSRect, NSSize};

use iced_wgpu::{wgpu, Backend, Renderer, Settings, Viewport};
use iced_winit::{futures, program, winit, Debug, Size};

use winit::event::{Event, ModifiersState, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::platform::desktop::EventLoopExtDesktop;
use winit::platform::macos::{ActivationPolicy, WindowBuilderExtMacOS, WindowExtMacOS};

use controls::Controls;

pub struct Editor {
    event_loop: EventLoop<()>,
    event_handler: RefCell<EventHandler>,
}

impl Editor {
    pub fn new() -> Self {
        let event_loop = EventLoop::new();
        let event_handler = RefCell::new(EventHandler::new(&event_loop));

        Self {
            event_loop,
            event_handler,
        }
    }

    #[cfg(target_os = "macos")]
    pub fn raw_view(&self) -> *mut c_void {
        self.event_handler.borrow().window.ns_view()
    }

    #[cfg(windows)]
    pub fn raw_view(&self) -> *mut c_void {
        // TODO
        std::ptr::null() as *mut c_void
    }

    #[cfg(target_os = "linux")]
    pub fn raw_view(&self) -> *mut c_void {
        std::ptr::null() as *mut c_void
    }

    pub fn open(&mut self) {
        let handler = self.event_handler.get_mut();
        handler.is_opened = true;
    }

    pub fn close(&mut self) {
        let handler = self.event_handler.get_mut();
        handler.is_opened = false;
    }

    pub fn event_loop_step(&mut self) {
        let handler = self.event_handler.get_mut();

        if !handler.is_opened {
            return;
        }

        self.event_loop
            .run_return(|event, _, control_flow| handler.handle(event, control_flow));
    }
}

struct EventHandler {
    window: winit::window::Window,
    viewport: Viewport,
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    format: wgpu::TextureFormat,
    swap_chain: wgpu::SwapChain,
    debug: Debug,
    renderer: Renderer,
    state: program::State<Controls>,
    modifiers: ModifiersState,
    is_resized: bool,
    is_opened: bool,
}

impl EventHandler {
    fn new(event_loop: &EventLoop<()>) -> Self {
        let window = winit::window::WindowBuilder::new()
            // .with_activation_policy(ActivationPolicy::Prohibited)
            .with_visible(true)
            .build(&event_loop)
            .unwrap();
        let viewport = Self::init_viewport(&window);

        let surface = wgpu::Surface::create(&window);
        let (mut device, queue) = Self::init_device_and_queue(&surface);
        let format = wgpu::TextureFormat::Bgra8UnormSrgb;
        let swap_chain = Self::init_swap_chain(&window, &device, &surface, &format);
        let mut debug = Debug::new();
        let mut renderer = Renderer::new(Backend::new(&mut device, Settings::default()));
        let state: program::State<Controls> = program::State::new(
            Controls::new(),
            viewport.logical_size(),
            &mut renderer,
            &mut debug,
        );

        EventHandler {
            window,
            viewport,
            surface,
            device,
            queue,
            format,
            swap_chain,
            debug,
            renderer,
            state,
            modifiers: Default::default(),
            is_resized: false,
            is_opened: true,
        }
    }

    fn init_viewport(window: &winit::window::Window) -> Viewport {
        let physical_size = window.inner_size();
        Viewport::with_physical_size(
            Size::new(physical_size.width, physical_size.height),
            window.scale_factor(),
        )
    }

    fn init_device_and_queue(surface: &wgpu::Surface) -> (wgpu::Device, wgpu::Queue) {
        futures::executor::block_on(async {
            let adapter = wgpu::Adapter::request(
                &wgpu::RequestAdapterOptions {
                    power_preference: wgpu::PowerPreference::Default,
                    compatible_surface: Some(&surface),
                },
                wgpu::BackendBit::PRIMARY,
            )
            .await
            .expect("Request adapter");

            adapter
                .request_device(&wgpu::DeviceDescriptor {
                    extensions: wgpu::Extensions {
                        anisotropic_filtering: false,
                    },
                    limits: wgpu::Limits::default(),
                })
                .await
        })
    }

    fn init_swap_chain(
        window: &winit::window::Window,
        device: &wgpu::Device,
        surface: &wgpu::Surface,
        format: &wgpu::TextureFormat,
    ) -> wgpu::SwapChain {
        let size = window.inner_size();

        device.create_swap_chain(
            &surface,
            &wgpu::SwapChainDescriptor {
                usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
                format: format.clone(),
                width: size.width,
                height: size.height,
                present_mode: wgpu::PresentMode::Mailbox,
            },
        )
    }

    fn handle(&mut self, event: Event<()>, control_flow: &mut ControlFlow) {
        match event {
            Event::WindowEvent { event, .. } => {
                match event {
                    WindowEvent::ModifiersChanged(new_modifiers) => {
                        self.modifiers = new_modifiers;
                    }
                    WindowEvent::Resized(new_size) => {
                        self.viewport = Viewport::with_physical_size(
                            Size::new(new_size.width, new_size.height),
                            self.window.scale_factor(),
                        );

                        self.is_resized = true;
                    }
                    WindowEvent::CloseRequested => {
                        self.is_opened = false;
                        *control_flow = ControlFlow::Exit;
                    }

                    _ => {}
                }

                // Map window event to iced event
                if let Some(event) = iced_winit::conversion::window_event(
                    &event,
                    self.window.scale_factor(),
                    self.modifiers,
                ) {
                    self.state.queue_event(event);
                }
            }
            Event::MainEventsCleared => {
                // We update iced
                let _ = self.state.update(
                    None,
                    self.viewport.logical_size(),
                    &mut self.renderer,
                    &mut self.debug,
                );

                // and request a redraw
                self.window.request_redraw();
            }
            Event::RedrawRequested(_) => {
                if self.is_resized {
                    let size = self.window.inner_size();

                    self.swap_chain = self.device.create_swap_chain(
                        &self.surface,
                        &wgpu::SwapChainDescriptor {
                            usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
                            format: self.format,
                            width: size.width,
                            height: size.height,
                            present_mode: wgpu::PresentMode::Mailbox,
                        },
                    );
                }

                let frame = self.swap_chain.get_next_texture().expect("Next frame");

                let mut encoder = self
                    .device
                    .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

                let _ = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                        attachment: &frame.view,
                        resolve_target: None,
                        load_op: wgpu::LoadOp::Clear,
                        store_op: wgpu::StoreOp::Store,
                        clear_color: wgpu::Color {
                            r: 1.0,
                            g: 0.5,
                            b: 0.0,
                            a: 1.0,
                        },
                    }],
                    depth_stencil_attachment: None,
                });

                // And then iced on top
                let mouse_interaction = self.renderer.backend_mut().draw(
                    &mut self.device,
                    &mut encoder,
                    &frame.view,
                    &self.viewport,
                    self.state.primitive(),
                    &self.debug.overlay(),
                );

                // Then we submit the work
                self.queue.submit(&[encoder.finish()]);

                // And update the mouse cursor
                self.window
                    .set_cursor_icon(iced_winit::conversion::mouse_interaction(mouse_interaction));
            }
            // we use Poll instead of Wait, because we can't pause the thread on Plugin::idle
            // and Plugin::idle does its own optimizations
            _ => *control_flow = ControlFlow::Poll,
        }
    }
}

// pub fn main() {
// let frame = NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(500.0, 400.0));
// let parent_window = unsafe {
// NSWindow::alloc(nil).initWithContentRect_styleMask_backing_defer_(
// frame,
// NSWindowStyleMask::NSBorderlessWindowMask | NSWindowStyleMask::NSTitledWindowMask,
// NSBackingStoreType::NSBackingStoreBuffered,
// 0,
// )
// };
// // this fixes mouse hover
// unsafe {
// parent_window.setAcceptsMouseMovedEvents_(1);
// };
//
// unsafe {
// let child = window.ns_view() as id;
// NSView::setFrameSize(child, frame.size);
// NSView::setFrameOrigin(child, frame.origin);
// parent_window.contentView().addSubview_(child);
// };
//
// // Initialize wgpu
//
// // Initialize GUI controls
//
// // Initialize iced
//
// unsafe { parent_window.orderFront_(parent_window) };
//
// let mut is_close = false;
//
// while self.is_opened {}
// }
