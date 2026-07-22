use criterion::{black_box, criterion_group, criterion_main, Criterion};
use libloadviz::renderer::Renderer;

fn rendering_busy(c: &mut Criterion) {
    let width = 100;
    let height = 100;
    let mut pixels = vec![0; width * height * 3];
    let renderer: Renderer = Default::default();

    let cpu_loads = vec![
        libloadviz::cpuload::CpuLoad {
            user_0_to_1: 0.1,
            system_0_to_1: 0.2,
        },
        libloadviz::cpuload::CpuLoad {
            user_0_to_1: 0.5,
            system_0_to_1: 0.5,
        },
    ];

    c.bench_function("render 100x100 image", |b| {
        b.iter(|| renderer.render_image(black_box(&cpu_loads), width, height, 0.0, &mut pixels));
    });
}

fn rendering_idle(c: &mut Criterion) {
    let width = 100;
    let height = 100;
    let mut pixels = vec![0; width * height * 3];
    let renderer: Renderer = Default::default();

    // A representative idle system: most cores quiet, a couple with light
    // background activity, one busier thread. Non-zero and heterogeneous so
    // this exercises the real per-pixel noise path, not the all-zero fast-out.
    let cpu_loads = vec![
        libloadviz::cpuload::CpuLoad {
            user_0_to_1: 0.0,
            system_0_to_1: 0.0,
        },
        libloadviz::cpuload::CpuLoad {
            user_0_to_1: 0.0,
            system_0_to_1: 0.0,
        },
        libloadviz::cpuload::CpuLoad {
            user_0_to_1: 0.0,
            system_0_to_1: 0.0,
        },
        libloadviz::cpuload::CpuLoad {
            user_0_to_1: 0.02,
            system_0_to_1: 0.01,
        },
        libloadviz::cpuload::CpuLoad {
            user_0_to_1: 0.03,
            system_0_to_1: 0.01,
        },
        libloadviz::cpuload::CpuLoad {
            user_0_to_1: 0.05,
            system_0_to_1: 0.02,
        },
        libloadviz::cpuload::CpuLoad {
            user_0_to_1: 0.15,
            system_0_to_1: 0.03,
        },
        libloadviz::cpuload::CpuLoad {
            user_0_to_1: 0.20,
            system_0_to_1: 0.05,
        },
    ];

    c.bench_function("render 100x100 image on an idle system", |b| {
        b.iter(|| renderer.render_image(black_box(&cpu_loads), width, height, 0.0, &mut pixels));
    });
}

criterion_group!(benches, rendering_busy, rendering_idle);
criterion_main!(benches);
