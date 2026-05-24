# runx CLI container image.
#
# The binary is pulled from the GitHub Release hub (the same musl archive every
# other channel consumes), so the image build needs no Rust toolchain and stays
# multi-arch without QEMU: the fetch stage runs on the build platform and selects
# the binary matching the requested TARGETARCH.

ARG VERSION

FROM --platform=$BUILDPLATFORM alpine:3 AS fetch
ARG VERSION
ARG TARGETARCH
RUN apk add --no-cache curl tar
RUN set -eux; \
    case "$TARGETARCH" in \
      amd64) target=x86_64-unknown-linux-musl ;; \
      arm64) target=aarch64-unknown-linux-musl ;; \
      *) echo "unsupported TARGETARCH: $TARGETARCH" >&2; exit 1 ;; \
    esac; \
    base="https://github.com/runxhq/runx/releases/download/cli-v${VERSION}"; \
    curl -fsSL "${base}/runx-${VERSION}-${target}.tar.gz" -o /tmp/runx.tgz; \
    tar -xzf /tmp/runx.tgz -C /tmp; \
    install -Dm755 "/tmp/runx-${VERSION}-${target}/runx" /out/runx

FROM gcr.io/distroless/static-debian12:nonroot
COPY --from=fetch /out/runx /usr/local/bin/runx
ENTRYPOINT ["/usr/local/bin/runx"]
