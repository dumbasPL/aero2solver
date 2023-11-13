FROM --platform=${BUILDPLATFORM:-linux/amd64} tonistiigi/xx AS xx

FROM --platform=${BUILDPLATFORM:-linux/amd64} rust:1-bullseye as builder

WORKDIR /app

COPY --from=xx / /

# install native dependencies first so we can cache them
RUN apt-get update && \
    apt-get install -y build-essential cmake clang llvm g++

ARG TARGETPLATFORM
ENV TARGETPLATFORM=${TARGETPLATFORM:-linux/amd64}

RUN xx-apt-get install -y xx-cxx-essentials g++

COPY . .

RUN ./build.sh

FROM debian:bullseye-slim AS runtime

WORKDIR /app

RUN apt-get update && \
    apt-get install -y libgomp1 && \
    rm -rf /var/lib/apt/lists/*

COPY ./model/ ./model/
COPY --from=builder /app/target/aero2solver ./aero2solver

ENTRYPOINT [ "/app/aero2solver" ]