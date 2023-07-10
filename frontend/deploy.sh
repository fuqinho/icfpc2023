#!/bin/bash

cd -- "$(dirname -- "${BASH_SOURCE[0]}")"
cd ..

set -ex

TAG="asia-northeast1-docker.pkg.dev/icfp-contest-compute/icfpc2023-frontend/icfpc2023-frontend:$(git rev-parse HEAD)"
docker build . -f ./frontend/Dockerfile -t "$TAG" --push
gcloud run deploy icfpc2023-frontend --image "$TAG" --region asia-northeast1 --project icfp-contest-compute
