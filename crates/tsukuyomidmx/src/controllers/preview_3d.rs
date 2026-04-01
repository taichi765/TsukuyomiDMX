use slint::{ComponentHandle, Image};
use wgpu::RenderPassColorAttachment;

use crate::AppWindow;

pub fn setup_3d_preview(ui: &AppWindow) {
    let mut renderer = None;
    let ui_handle = ui.as_weak();

    ui.window()
        .set_rendering_notifier(move |state, graphics_api| match state {
            slint::RenderingState::RenderingSetup => {
                match graphics_api {
                    slint::GraphicsAPI::WGPU27 { device, queue, .. } => {
                        renderer = Some(PreviewRenderer::new(device, queue));
                    }
                    _ => return,
                };
            }
            slint::RenderingState::BeforeRendering => {
                if let Some(renderer) = renderer.as_mut() {
                    let texture = renderer.render();
                    let ui = ui_handle.unwrap();
                    ui.set_texture(Image::try_from(texture).unwrap());
                    ui.window().request_redraw();
                }
            }
            slint::RenderingState::AfterRendering => {}
            slint::RenderingState::RenderingTeardown => {
                drop(renderer.take());
            }
            _ => {}
        })
        .expect("unable to set rendering notifier");
}

struct PreviewRenderer {
    device: wgpu::Device,
    queue: wgpu::Queue,
    texture: wgpu::Texture,
}

impl PreviewRenderer {
    fn new(device: &wgpu::Device, queue: &wgpu::Queue) -> Self {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: None,
            size: wgpu::Extent3d {
                width: 500, //TODO: take parameter, dynamic
                height: 300,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        Self {
            device: device.clone(),
            queue: queue.clone(),
            texture,
        }
    }

    fn render(&mut self) -> wgpu::Texture {
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });
        {
            let _rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: &self
                        .texture
                        .create_view(&wgpu::TextureViewDescriptor::default()),
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: 0.2,
                            b: 0.3,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
        }
        self.queue.submit(std::iter::once(encoder.finish()));

        self.texture.clone() //FIXME: unneccesary clone?
    }
}
