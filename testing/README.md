# Testing

an arch environment with pending updates for testing pacfetch -Syu/-Sy/-Su 

## Requirements

- Docker

## Usage

cd testing

./test.sh

./test.sh -Syu

./test.sh -Su

./test.sh -Sy


### Or drop into shell
./test.sh bash

## Updating the test image

The Dockerfile uses a dated Arch Linux image. To refresh it:

1. Find a recent tag at https://hub.docker.com/_/archlinux/tags
2. Update `ARCH_TAG` in the Dockerfile
3. Remove old image: `docker rmi pacfetch-test`
4. Re-run `./test.sh` to rebuild
