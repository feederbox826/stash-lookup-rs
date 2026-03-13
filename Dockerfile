FROM scratch
ARG TARGETARCH

COPY --chmod=555 docker/${TARGETARCH}/stash-lookup /stash-lookup

ENTRYPOINT ["/stash-lookup"]