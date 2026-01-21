# arch-env.sh

An Arch environment with pending updates for testing pacfetch -Syu/-Sy/-Su.

## Requirements

- Docker

## Usage 

### Drop into shell and test
```
./arch-env.sh bash
```

### Or run directly
```
./arch-env.sh -Syu
./arch-env.sh -Sy
./arch-env.sh -Su
```

### Updating the test image

The Dockerfile uses a dated Arch Linux image. To refresh it:

1. Find a recent tag at https://hub.docker.com/_/archlinux/tags
2. Update `ARCH_TAG` in the Dockerfile
3. Remove old image: `docker rmi pacfetch-test`
4. Re-run `./arch-env.sh` to rebuild

----

# smoke-test.sh

Runs all flag combinations and verifies exit codes. Failures are logged to `smoke-test.log`.

```
./smoke-test.sh
```
