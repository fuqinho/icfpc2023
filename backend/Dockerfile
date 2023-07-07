FROM golang:1.20 as go

WORKDIR /build

COPY go.mod go.sum ./
RUN go mod download

COPY . ./
RUN CGO_ENABLED=0 go install icfpc2023/backend

FROM ubuntu:latest

WORKDIR /app
COPY --from=go /go/bin/backend .
COPY --from=go /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/

RUN useradd -M app
ENV HOME=/app
USER app

ENTRYPOINT ["./backend"]
