FROM fuzzers/cargo-fuzz:0.11.0 as builder

ADD . /h3o
WORKDIR /h3o

RUN cd fuzz/ && cargo fuzz build

FROM ubuntu:22.04

COPY --from=builder /h3o/fuzz/target/x86_64-unknown-linux-gnu/release/cell_index /
COPY --from=builder /h3o/fuzz/target/x86_64-unknown-linux-gnu/release/cell_index_hierarchy /
COPY --from=builder /h3o/fuzz/target/x86_64-unknown-linux-gnu/release/cell_index_pair /
COPY --from=builder /h3o/fuzz/target/x86_64-unknown-linux-gnu/release/cell_to_vertex /
COPY --from=builder /h3o/fuzz/target/x86_64-unknown-linux-gnu/release/cells_to_geom /
COPY --from=builder /h3o/fuzz/target/x86_64-unknown-linux-gnu/release/compact /
COPY --from=builder /h3o/fuzz/target/x86_64-unknown-linux-gnu/release/directed_edge_index /
COPY --from=builder /h3o/fuzz/target/x86_64-unknown-linux-gnu/release/grid /
COPY --from=builder /h3o/fuzz/target/x86_64-unknown-linux-gnu/release/index_io /
COPY --from=builder /h3o/fuzz/target/x86_64-unknown-linux-gnu/release/is_valid /
COPY --from=builder /h3o/fuzz/target/x86_64-unknown-linux-gnu/release/latltng_to_cell /
COPY --from=builder /h3o/fuzz/target/x86_64-unknown-linux-gnu/release/polygon_to_cells /
COPY --from=builder /h3o/fuzz/target/x86_64-unknown-linux-gnu/release/polygon_with_holes_to_cells /
COPY --from=builder /h3o/fuzz/target/x86_64-unknown-linux-gnu/release/uncompact /
COPY --from=builder /h3o/fuzz/target/x86_64-unknown-linux-gnu/release/vertex_index /
