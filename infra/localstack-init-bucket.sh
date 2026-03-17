#!/usr/bin/env sh
set -eu

bucket_name="${S3_BUCKET_NAME:-hdrop}"

if awslocal s3api list-buckets --query 'Buckets[].Name' --output text | tr '\t' '\n' | grep -Fxq "${bucket_name}"; then
  exit 0
fi

awslocal s3 mb "s3://${bucket_name}"
