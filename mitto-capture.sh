#!/usr/bin/env bash
#
# Utility script for capturing a screenshot with Flameshot and uploading it to a Mitto instance.
# Dependencies: flameshot, curl, file, notify-send, wl-clipboard
#
# Based on:
#  - https://github.com/TannerReynolds/ShareX-Upload-Server?tab=readme-ov-file#using-with-flameshot-linux
#  - https://github.com/flameshot-org/flameshot/issues/178
#
# Usage:
# MITTO_URL=https://my-host.com/upload MITTO_USER=myuser MITTO_PASS=mypass ./mitto-capture.sh

function _notify() {
    notify-send --expire-time 2000 --app-name "Flameshot" --icon "$1" "Screenshot" "$2"
}

tmp_file=$(mktemp --dry-run -t "screen_$(date +'%Y%m%d')_XXXXXX.png")

echo "Taking screenshot and saving it to '$tmp_file'"
flameshot gui -r > "$tmp_file"

if [[ $(file --mime-type -b "$tmp_file") != "image/png" ]]; then
	rm "$tmp_file"
	_notify "flameshot" "Screenshot aborted"
	exit 1
fi

upload_url=$(./mitto-upload.sh "$tmp_file")
if [ $? -ne 0 ]; then
	wl-copy < "$tmp_file"
	_notify "flameshot" "Upload failed - $upload_url"
	exit 2
fi

wl-copy "$upload_url"
_notify "$tmp_file" "Uploaded! $upload_url"
rm "$tmp_file"
exit 0
