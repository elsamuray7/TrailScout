FROM rust:1.62 as build

#copy could be more specific but not that important unless target already exsits in which case you should probably delete it before building
COPY . .
RUN cargo build --release

FROM debian:11.4-slim
# copy the build artifact from the build stage
COPY --from=build ./target/release/trailscout_service /backend/
COPY ./osm_graphs /backend/osm_graphs
COPY config.json /backend/config.json
COPY sights_config.json /backend/sights_config.json
COPY edge_type_config.json /backend/edge_type_config.json
COPY preprocess_osm.py /backend/preprocess_osm.py

#setup some extra packages
RUN apt-get update
RUN apt-get -y install python3
RUN apt-get -y install procps
RUN apt-get -y install osmium-tool


EXPOSE 8080

WORKDIR backend

# set the startup command to run your binary
CMD ["./trailscout_service"]