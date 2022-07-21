FROM rust:latest as build

COPY . .
RUN cargo build --release

# our final base
FROM debian:buster-slim
# copy the build artifact from the build stage
COPY --from=build ./target/release/trailscout_service ./
COPY ./osm_graphs ./osm_graphs
COPY config.json config.json
COPY sights_config.json sights_config.json

EXPOSE 8080

# set the startup command to run your binary
CMD ["./trailscout_service"]