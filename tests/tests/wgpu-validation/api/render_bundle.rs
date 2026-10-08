//! Tests of [`wgpu::RenderBundleEncoder`] error reporting.

fn bundle_encoder(device: &wgpu::Device) -> wgpu::RenderBundleEncoder<'_> {
    device.create_render_bundle_encoder(&wgpu::RenderBundleEncoderDescriptor {
        label: Some("bundle"),
        color_formats: &[Some(wgpu::TextureFormat::Rgba8Unorm)],
        depth_stencil: None,
        sample_count: 1,
        multiview: None,
    })
}

/// A bundle that names an invalid buffer (for example one whose allocation
/// failed) reports an error through the error scopes instead of panicking,
/// and so does a render pass executing it.
#[test]
fn finish_with_invalid_buffer_is_an_error() {
    let (device, _queue) = wgpu::Device::noop(&wgpu::DeviceDescriptor::default());

    let scope = device.push_error_scope(wgpu::ErrorFilter::Validation);
    let buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: None,
        size: 1 << 63, // Too large: the buffer is invalid.
        usage: wgpu::BufferUsages::VERTEX,
        mapped_at_creation: false,
    });
    assert!(pollster::block_on(scope.pop()).is_some());

    let scope = device.push_error_scope(wgpu::ErrorFilter::Validation);
    let mut encoder = bundle_encoder(&device);
    encoder.set_vertex_buffer(0, buffer.slice(..));
    let bundle = encoder.finish(&wgpu::RenderBundleDescriptor {
        label: Some("bundle"),
    });
    let error = pollster::block_on(scope.pop()).expect("finish should report an error");
    assert!(
        error.to_string().contains("RenderBundleEncoder::finish"),
        "{error}"
    );

    // Executing the invalid bundle is an error, not a panic, as well.
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: None,
        size: wgpu::Extent3d {
            width: 4,
            height: 4,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8Unorm,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        view_formats: &[],
    });
    let view = texture.create_view(&Default::default());
    let scope = device.push_error_scope(wgpu::ErrorFilter::Validation);
    let mut command_encoder = device.create_command_encoder(&Default::default());
    {
        let mut pass = command_encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &view,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations::default(),
            })],
            ..Default::default()
        });
        pass.execute_bundles([&bundle]);
    }
    let _ = command_encoder.finish();
    assert!(pollster::block_on(scope.pop()).is_some());
}

/// Finishing a bundle on a lost device does not panic.
#[test]
fn finish_on_destroyed_device_does_not_panic() {
    let (device, _queue) = wgpu::Device::noop(&wgpu::DeviceDescriptor::default());
    device.destroy();

    let scope = device.push_error_scope(wgpu::ErrorFilter::Validation);
    let mut encoder = bundle_encoder(&device);
    encoder.draw(0..3, 0..1);
    let _bundle = encoder.finish(&wgpu::RenderBundleDescriptor { label: None });
    // A lost device reports no validation errors.
    assert!(pollster::block_on(scope.pop()).is_none());
}
