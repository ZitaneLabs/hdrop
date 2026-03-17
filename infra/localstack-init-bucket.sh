#!/usr/bin/env sh
set -eu

awslocal s3 mb s3://hdrop || true
