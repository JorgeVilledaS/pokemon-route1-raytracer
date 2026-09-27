#!/usr/bin/env sh
set -eu
ffmpeg -framerate 30 -i output/frames/frame_%04d.ppm -c:v libx264 -pix_fmt yuv420p output/final_video.mp4
