#### Protoc binary fetcher ####
FROM alpine:latest AS protocfetcher

RUN mkdir -p /proto

RUN apk add --no-cache wget unzip

ARG PROTOC_VERSION=3.12.4
RUN wget https://github.com/protocolbuffers/protobuf/releases/download/v${PROTOC_VERSION}/protoc-${PROTOC_VERSION}-linux-x86_64.zip
RUN unzip protoc-${PROTOC_VERSION}-linux-x86_64.zip -d /proto


#### Build server exec ####
FROM rust:latest AS serverbuilder

RUN mkdir -p /bge
WORKDIR /bge

COPY --from=protocfetcher /proto/bin/* /usr/local/bin/
COPY --from=protocfetcher /proto/include/* /usr/local/include/

COPY shared/ shared/
COPY server/ server/

WORKDIR /bge/server
RUN cargo build --release


#### Build FE staticfiles ####
FROM node:latest AS viewerbuilder

RUN mkdir -p /bge
WORKDIR /bge

COPY --from=protocfetcher /proto/bin/* /usr/local/bin/
COPY --from=protocfetcher /proto/include/* /usr/local/include/

COPY shared/ shared/
COPY viewer/ viewer/

WORKDIR /bge/viewer
RUN yarn install
RUN yarn gen-protos
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
