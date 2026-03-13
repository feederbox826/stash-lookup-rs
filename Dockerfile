FROM scratch
ARG TARGETARCH

COPY docker/${TARGETARCH}/stash-lookup /stash-lookup

ENTRYPOINT ["/stash-lookup"]