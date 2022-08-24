# Build Lambda Docker image
FROM amazonlinux:2 AS builder

WORKDIR /app

RUN yum install -y postgresql-devel openssl-devel && yum groupinstall -y "Development Tools"

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- --profile minimal --default-toolchain stable -y && \
    source ~/.cargo/env

ADD Cargo.toml .
ADD Cargo.lock .

RUN source ~/.cargo/env && cargo fetch

ARG COMMIT=""
ARG NOW=""
ENV KV_SERVER_BUILD_AT=${NOW}
ENV KV_SERVER_CURRENT_COMMIT_ID=${COMMIT}

ADD . .
RUN source ~/.cargo/env && cargo build --release --example lambda

# =-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-
FROM public.ecr.aws/lambda/provided:al2 AS runner
LABEL maintainer="Nyk Ma <nykma@mask.io>"

WORKDIR /app

RUN yum install -y postgresql-devel openssl-devel && yum clean all && rm -rf /var/cache/yum

COPY --from=builder /app/target/release/examples/lambda ${LAMBDA_RUNTIME_DIR}/bootstrap
