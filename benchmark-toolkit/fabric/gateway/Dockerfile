FROM golang:1.23-bookworm AS builder

WORKDIR /src
COPY go.mod ./
RUN go mod tidy
COPY . .
RUN go mod tidy
RUN CGO_ENABLED=0 go build -o /fabric-benchmark-gateway .

FROM gcr.io/distroless/static-debian12:nonroot
WORKDIR /app
COPY --from=builder /fabric-benchmark-gateway /app/fabric-benchmark-gateway
EXPOSE 8800
ENTRYPOINT ["/app/fabric-benchmark-gateway"]
