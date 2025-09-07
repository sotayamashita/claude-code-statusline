use claude_code_statusline_core::{Config, Engine, parse_claude_input};
use criterion::{Criterion, criterion_group, criterion_main};

fn bench_engine_render(c: &mut Criterion) {
    let cfg = Config::default();
    let engine = Engine::new(cfg);
    let json = r#"{
        "session_id": "bench",
        "cwd": "/tmp",
        "model": {"id": "claude-opus", "display_name": "Opus"}
    }"#;
    let input = parse_claude_input(json).unwrap();

    c.bench_function("engine_render_default", |b| {
        b.iter(|| {
            let _ = engine.render(&input).unwrap();
        })
    });
}

criterion_group!(benches, bench_engine_render);
criterion_main!(benches);
