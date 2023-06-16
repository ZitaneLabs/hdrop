#!/usr/bin/env bash

DATABASE_URL=postgres://postgres:postgres@localhost:5432/hdrop

pushd hdrop-db
diesel migration run --database-url $DATABASE_URL
popd
