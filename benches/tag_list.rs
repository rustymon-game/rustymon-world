use criterion::{criterion_group, criterion_main, Criterion};
use libosmium::{
    tag_list,
    tag_list::{OwnedTagList, TagIterator},
};
use std::collections::HashSet;

fn check_by_iteration<'a>(
    tag_list: impl IntoIterator<IntoIter = TagIterator<'a>>,
    keys: &'a HashSet<&str>,
) -> bool {
    for (key, _) in tag_list.into_iter() {
        if keys.contains(key) {
            return true;
        }
    }
    false
}

fn check_by_intersection<'a>(
    tag_list: impl IntoIterator<IntoIter = TagIterator<'a>>,
    keys: &'a HashSet<&str>,
) -> bool {
    let tag_keys = HashSet::from_iter(tag_list.into_iter().map(|(key, _)| key));
    !keys.is_disjoint(&tag_keys)
}

fn check_with_recycling<'a>(
    tag_list: impl IntoIterator<IntoIter = TagIterator<'a>> + 'a,
    recycle: &'a mut HashSet<&'static str>,
    keys: &'a HashSet<&'a str>,
) -> bool {
    recycle.extend(
        tag_list
            .into_iter()
            .map(|(key, _)| unsafe { std::mem::transmute::<&'a str, &'static str>(key) }),
    );
    let ret = !keys.is_disjoint(recycle);
    recycle.clear();
    return ret;
}

fn test(c: &mut Criterion) {
    let expected_keys = HashSet::from_iter(["foo", "bar", "baz"]);
    let tag_list: OwnedTagList = tag_list! {"name": "Bob", "age": "12", "foo": ""};

    assert!(check_by_iteration(&tag_list, &expected_keys));
    assert!(check_by_intersection(&tag_list, &expected_keys));

    c.bench_function("check_by_iteration", |b| {
        b.iter(|| {
            check_by_iteration(&tag_list, &expected_keys);
        })
    });
    c.bench_function("check_by_intersection", |b| {
        b.iter(|| {
            check_by_intersection(&tag_list, &expected_keys);
        })
    });
    let mut set = HashSet::new();
    c.bench_function("check_with_recycling", |b| {
        b.iter({
            || {
                check_with_recycling(&tag_list, &mut set, &expected_keys);
            }
        })
    });
}

criterion_group!(benches, test);
criterion_main!(benches);
