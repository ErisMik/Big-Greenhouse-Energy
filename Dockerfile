#### Build server exec ####
FROM rust:latest AS serverbuilder

RUN mkdir -p /bge/server
WORKDIR /bge/server

COPY server/ .

RUN cargo build --release


#### Build FE staticfiles ####
FROM node:latest AS viewerbuilder

RUN mkdir -p /bge/viewer
WORKDIR /bge/viewer

COPY viewer/ .

RUN yarn install
RUN yarn build


#### Host with nginx ####
FROM nginx:latest AS thehost

RUN mkdir -p /bge
WORKDIR /bge

EXPOSE 3030

COPY --from=serverbuilder /bge/server/target/release/bge_server /bge/server/
COPY --from=viewerbuilder /bge/viewer/build/ /bge/viewer/build/

WORKDIR /bge/server
CMD "/bge/server/bge_server"
