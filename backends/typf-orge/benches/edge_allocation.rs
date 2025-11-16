// this_file: backends/typf-orge/benches/edge_allocation.rs

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use typf_orge::edge::{Edge, EdgeList};
use typf_orge::fixed::F26Dot6;

fn bench_vec_allocation(c: &mut Criterion) {
    c.bench_function("edgelist_vec_push_100", |b| {
        b.iter(|| {
            let mut list = EdgeList::new();
            for i in 0..100 {
                if let Some(edge) = Edge::new(
                    F26Dot6::from_int(i),
                    F26Dot6::from_int(0),
                    F26Dot6::from_int(i + 10),
                    F26Dot6::from_int(10),
                ) {
                    list.push(edge);
                }
            }
            black_box(list);
        });
    });

    c.bench_function("edgelist_vec_with_capacity_100", |b| {
        b.iter(|| {
            let mut list = EdgeList::with_capacity(100);
            for i in 0..100 {
                if let Some(edge) = Edge::new(
                    F26Dot6::from_int(i),
                    F26Dot6::from_int(0),
                    F26Dot6::from_int(i + 10),
                    F26Dot6::from_int(10),
                ) {
                    list.push(edge);
                }
            }
            black_box(list);
        });
    });

    c.bench_function("edgelist_insert_sorted_100", |b| {
        b.iter(|| {
            let mut list = EdgeList::new();
            for i in (0..100).rev() {
                if let Some(edge) = Edge::new(
                    F26Dot6::from_int(i),
                    F26Dot6::from_int(0),
                    F26Dot6::from_int(i + 10),
                    F26Dot6::from_int(10),
                ) {
                    list.insert_sorted(edge);
                }
            }
            black_box(list);
        });
    });

    c.bench_function("edgelist_sort_by_x_100", |b| {
        let mut list = EdgeList::new();
        for i in (0..100).rev() {
            if let Some(edge) = Edge::new(
                F26Dot6::from_int(i),
                F26Dot6::from_int(0),
                F26Dot6::from_int(i + 10),
                F26Dot6::from_int(10),
            ) {
                list.push(edge);
            }
        }
        b.iter(|| {
            let mut list_copy = list.clone();
            list_copy.sort_by_x();
            black_box(list_copy);
        });
    });
}

criterion_group!(benches, bench_vec_allocation);
criterion_main!(benches);
