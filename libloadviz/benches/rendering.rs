use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn criterion_benchmark(c: &mut Criterion) {
    let width = 100;
    let height = 100;
    let mut pixels = vec![0; width * height * 3];

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
        b.iter(|| {
            libloadviz::renderer::render_image_raw(
                black_box(&cpu_loads),
                width,
                height,
                &mut pixels,
            )
        });
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
