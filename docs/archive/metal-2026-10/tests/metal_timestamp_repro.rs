//! Diagnostic: compare timestamp writes across command encoders on the GPU.
//! Run explicitly with `cargo test --workspace metal_timestamp_repro -- --ignored --nocapture`.

#[test]
#[ignore]
fn metal_timestamp_repro() {
    let instance = wgpu::Instance::default();
    let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions::default())).expect("adapter");
    if !adapter.features().contains(wgpu::Features::TIMESTAMP_QUERY) {
        eprintln!("timestamp queries unsupported");
        return;
    }
    let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
        required_features: wgpu::Features::TIMESTAMP_QUERY,
        ..Default::default()
    }))
    .expect("device");
    eprintln!("adapter: {:?}, period: {} ns", adapter.get_info(), queue.get_timestamp_period());
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: None,
        size: wgpu::Extent3d {
            width: 64,
            height: 64,
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
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: None, source: wgpu::ShaderSource::Wgsl("@vertex fn vs(@builtin(vertex_index) i: u32) -> @builtin(position) vec4f { let x = f32((i << 1u) & 2u); let y = f32(i & 2u); return vec4f(x * 2.0 - 1.0, y * 2.0 - 1.0, 0.0, 1.0); } @fragment fn fs() -> @location(0) vec4f { return vec4f(1.0, 0.5, 0.1, 1.0); }".into()),
    });
    let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: None,
        layout: None,
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: Some("vs"),
            buffers: &[],
            compilation_options: Default::default(),
        },
        primitive: Default::default(),
        depth_stencil: None,
        multisample: Default::default(),
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: Some("fs"),
            targets: &[Some(wgpu::ColorTargetState {
                format: wgpu::TextureFormat::Rgba8Unorm,
                blend: None,
                write_mask: wgpu::ColorWrites::ALL,
            })],
            compilation_options: Default::default(),
        }),
        multiview_mask: None,
        cache: None,
    });
    for (same_pass, split, parallel, fresh) in [
        (true, false, false, false),
        (true, false, false, true),
        (false, false, false, true),
        (false, true, true, true),
    ] {
        let query = device.create_query_set(&wgpu::QuerySetDescriptor {
            label: None,
            ty: wgpu::QueryType::Timestamp,
            count: 2,
        });
        let resolve = device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: 16,
            usage: wgpu::BufferUsages::QUERY_RESOLVE | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });
        let readback = device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: 16,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let mut invalid = 0;
        let mut backwards = 0;
        let mut last = 0;
        for _ in 0..200 {
            let fresh_query = fresh.then(|| {
                device.create_query_set(&wgpu::QuerySetDescriptor {
                    label: None,
                    ty: wgpu::QueryType::Timestamp,
                    count: 2,
                })
            });
            let query = fresh_query.as_ref().unwrap_or(&query);
            let mut first = device.create_command_encoder(&Default::default());
            let pass = |encoder: &mut wgpu::CommandEncoder, begin, end| {
                let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: None,
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &view,
                        depth_slice: None,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                            store: wgpu::StoreOp::Store,
                        },
                    })],
                    depth_stencil_attachment: None,
                    timestamp_writes: Some(wgpu::RenderPassTimestampWrites {
                        query_set: query,
                        beginning_of_pass_write_index: begin,
                        end_of_pass_write_index: end,
                    }),
                    occlusion_query_set: None,
                    multiview_mask: None,
                });
                pass.set_pipeline(&pipeline);
                pass.draw(0..3, 0..1);
            };
            pass(&mut first, Some(0), same_pass.then_some(1));
            let (pre, mut second) = if split {
                (Some(first), device.create_command_encoder(&Default::default()))
            } else {
                (None, first)
            };
            if !same_pass {
                pass(&mut second, None, Some(1));
            }
            second.resolve_query_set(query, 0..2, &resolve, 0);
            second.copy_buffer_to_buffer(&resolve, 0, &readback, 0, 16);
            let commands = if let Some(pre) = pre {
                if parallel {
                    let (a, b) = std::thread::scope(|scope| {
                        let handle = scope.spawn(move || pre.finish());
                        let b = second.finish();
                        (handle.join().expect("finish"), b)
                    });
                    vec![a, b]
                } else {
                    vec![pre.finish(), second.finish()]
                }
            } else {
                vec![second.finish()]
            };
            queue.submit(commands);
            let (tx, rx) = std::sync::mpsc::channel();
            readback.map_async(wgpu::MapMode::Read, .., move |result| {
                let _ = tx.send(result);
            });
            device.poll(wgpu::PollType::wait_indefinitely()).expect("poll");
            rx.recv().expect("map callback").expect("map");
            let data = readback.get_mapped_range(..).expect("mapped");
            let ts: &[u64] = bytemuck::cast_slice(&data);
            if ts[1] <= ts[0] || ts[0] == 0 {
                invalid += 1;
                if invalid <= 3 {
                    eprintln!("invalid: begin={} end={}", ts[0], ts[1]);
                }
            }
            if ts[0] <= last {
                backwards += 1;
            }
            last = ts[0];
            drop(data);
            readback.unmap();
        }
        eprintln!(
            "same_pass={same_pass} split={split} parallel={parallel} fresh={fresh} invalid={invalid}/200 nonprogressing={backwards}/200"
        );
    }
}

#[test]
#[ignore]
fn metal_timestamp_compute_repro() {
    let instance = wgpu::Instance::default();
    let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions::default())).expect("adapter");
    if !adapter.features().contains(wgpu::Features::TIMESTAMP_QUERY) {
        return;
    }
    let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
        required_features: wgpu::Features::TIMESTAMP_QUERY,
        ..Default::default()
    }))
    .expect("device");
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: None,
        source: wgpu::ShaderSource::Wgsl("@compute @workgroup_size(1) fn main() {}".into()),
    });
    let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: None,
        layout: None,
        module: &shader,
        entry_point: Some("main"),
        compilation_options: Default::default(),
        cache: None,
    });
    for fresh in [false, true] {
        let shared = device.create_query_set(&wgpu::QuerySetDescriptor {
            label: None,
            ty: wgpu::QueryType::Timestamp,
            count: 2,
        });
        let resolve = device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: 16,
            usage: wgpu::BufferUsages::QUERY_RESOLVE | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });
        let readback = device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: 16,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let mut invalid = 0;
        let mut nonprogressing = 0;
        let mut previous = (0, 0);
        for _ in 0..200 {
            let own = fresh.then(|| {
                device.create_query_set(&wgpu::QuerySetDescriptor {
                    label: None,
                    ty: wgpu::QueryType::Timestamp,
                    count: 2,
                })
            });
            let query = own.as_ref().unwrap_or(&shared);
            let mut encoder = device.create_command_encoder(&Default::default());
            {
                let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                    label: None,
                    timestamp_writes: Some(wgpu::ComputePassTimestampWrites {
                        query_set: query,
                        beginning_of_pass_write_index: Some(0),
                        end_of_pass_write_index: Some(1),
                    }),
                });
                pass.set_pipeline(&pipeline);
                pass.dispatch_workgroups(1, 1, 1);
            }
            encoder.resolve_query_set(query, 0..2, &resolve, 0);
            encoder.copy_buffer_to_buffer(&resolve, 0, &readback, 0, 16);
            queue.submit([encoder.finish()]);
            let (tx, rx) = std::sync::mpsc::channel();
            readback.map_async(wgpu::MapMode::Read, .., move |r| {
                let _ = tx.send(r);
            });
            device.poll(wgpu::PollType::wait_indefinitely()).expect("poll");
            rx.recv().expect("callback").expect("map");
            let data = readback.get_mapped_range(..).expect("mapped");
            let ts: &[u64] = bytemuck::cast_slice(&data);
            if ts[0] == 0 || ts[1] <= ts[0] {
                invalid += 1;
                if invalid <= 3 {
                    eprintln!("invalid compute: {} {}", ts[0], ts[1]);
                }
            }
            if ts[0] <= previous.0 || ts[1] <= previous.1 {
                nonprogressing += 1;
            }
            previous = (ts[0], ts[1]);
            drop(data);
            readback.unmap();
        }
        eprintln!("compute fresh={fresh} invalid={invalid}/200 nonprogressing={nonprogressing}/200");
    }
}
