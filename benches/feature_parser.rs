use criterion::{criterion_group, criterion_main, BatchSize, BenchmarkId, Criterion};
use libosmium::tag_list::OwnedTagList;
use rustymon_world::features;
use rustymon_world::features::FeatureParser;
use serde::Deserialize;

#[derive(Deserialize, Default)]
pub struct Samples {
    pub size: usize,
    pub areas: Vec<OwnedTagList>,
    pub nodes: Vec<OwnedTagList>,
    pub ways: Vec<OwnedTagList>,
}

fn random_elem<'s, T>(slice: &'s [T]) -> impl FnMut() -> &'s T + 's {
    let mut index = slice.len();
    move || {
        index += 1;
        if index >= slice.len() {
            index = 0;
        }
        &slice[index]
    }
}

fn compare(c: &mut Criterion) {
    let dir = std::env::current_dir().unwrap();

    // Load tags
    let tags = std::fs::File::open(dir.join("../tags.msgpack")).unwrap();
    let tags: Samples = rmp_serde::from_read(tags).unwrap();

    // Load parser
    let config = dir.join("../visual.config");

    let yada = features::yada::YadaParser::from_file(&config).unwrap();
    // TODO more parser prototypes

    let mut group = c.benchmark_group("Feature Parser");

    macro_rules! dynamic_for {
        (parser in [$($parser:expr),+]) => {$(
            for (name, slice) in [("areas", &tags.areas), ("nodes", &tags.nodes), ("ways", &tags.ways)] {
                group.bench_with_input(BenchmarkId::new(stringify!($parser), name), slice, |b, slice| {
                    b.iter_batched(
                        random_elem(slice),
                        |tags| $parser.way(tags),
                        BatchSize::SmallInput,
                    );
                });
            }
        )+};
    }
    dynamic_for!(parser in [yada]);

    group.finish();
}

criterion_group!(benches, compare);
criterion_main!(benches);
