#!/usr/bin/env bash
#
# Utility script for uploading a file to a Mitto instance.
# Dependencies: curl
#
# Usage:
# MITTO_URL=https://my-host.com/upload MITTO_USER=myuser MITTO_PASS=mypass ./mitto-upload.sh FILE

function echoerr() {
  echo "$@" 1>&2;
}

file=$1
if [ -z "$file" ]; then
  echoerr "Error: missing file argument"
  exit 1
fi
if ! [ -f "$file" ]; then
  echoerr "Error: input file does not exist: $file"
  exit 2
fi

url=${MITTO_URL:-http://127.0.0.1:8080/upload}
auth=${MITTO_USER:-}:${MITTO_PASS:-}

echoerr "Uploading '$file' to '$url'"
upload_url=$(curl "$url" --request POST --form "file=@$file" --basic --user "$auth" --fail --location --silent --show-error --stderr -)
retval=$?

if [ $retval -ne 0 ]; then
  echoerr "Error: upload failed with: $upload_url"
  exit 3
fi

echo -n "$upload_url"
exit 0
