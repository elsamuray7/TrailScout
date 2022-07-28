FROM rust:latest as build

#copy could be more specific but not that important unless target already exsits in which case you should probably delete it before building
COPY . .
RUN cargo build --release

FROM debian:buster-slim
# copy the build artifact from the build stage
COPY --from=build ./target/release/trailscout_service ./
COPY ./osm_graphs ./osm_graphs
COPY config.json config.json
COPY sights_config.json sights_config.json

EXPOSE 8080

# set the startup command to run your binary
CMD ["./trailscout_service"]