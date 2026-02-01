#!/bin/bash
# runs all flag combinations

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
BIN="$PROJECT_DIR/target/release/pacfetch"
LOG="$SCRIPT_DIR/smoke-test.log"

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

PASS=0
FAIL=0

echo "" >> "$LOG"
echo "================================" >> "$LOG"
echo "Run: $(date '+%Y-%m-%d %H:%M:%S')" >> "$LOG"
echo "================================" >> "$LOG"

log_fail() {
    echo "----------------------------------------" >> "$LOG"
    echo "FAILED: $1" >> "$LOG"
    echo "Command: $2" >> "$LOG"
    echo "Output:" >> "$LOG"
    echo "$3" >> "$LOG"
    echo "" >> "$LOG"
}

echo "Building pacfetch..."
cargo build --release --manifest-path "$PROJECT_DIR/Cargo.toml" 2>/dev/null

# Test exits 0
test_ok() {
    local name="$1"
    shift
    local cmd="$*"

    echo -en "${YELLOW}$name${NC} ... "

    local output
    local exit_code
    output=$("$@" 2>&1) && exit_code=0 || exit_code=$?

    if [[ $exit_code -eq 0 ]]; then
        echo -e "${GREEN}PASS${NC}"
        PASS=$((PASS + 1))
    else
        echo -e "${RED}FAIL${NC}"
        log_fail "$name" "$cmd" "Exit code: $exit_code"$'\n'"$output"
        FAIL=$((FAIL + 1))
    fi
}

# Test exits non 0
test_err() {
    local name="$1"
    shift
    local cmd="$*"

    echo -en "${YELLOW}$name${NC} ... "

    local output
    local exit_code
    output=$("$@" 2>&1) && exit_code=0 || exit_code=$?

    if [[ $exit_code -ne 0 ]]; then
        echo -e "${GREEN}PASS${NC}"
        PASS=$((PASS + 1))
    else
        echo -e "${RED}FAIL${NC}"
        log_fail "$name" "$cmd" "Expected error but got exit 0"$'\n'"$output"
        FAIL=$((FAIL + 1))
    fi
}

# Test prompt w piped n succeeds
test_ok_piped() {
    local name="$1"
    local input="$2"
    shift 2
    local cmd="$*"

    echo -en "${YELLOW}$name${NC} ... "

    local output
    local exit_code
    output=$(echo "$input" | "$@" 2>&1) && exit_code=0 || exit_code=$?

    if [[ $exit_code -eq 0 ]]; then
        echo -e "${GREEN}PASS${NC}"
        PASS=$((PASS + 1))
    else
        echo -e "${RED}FAIL${NC}"
        log_fail "$name" "echo '$input' | $cmd" "Exit code: $exit_code"$'\n'"$output"
        FAIL=$((FAIL + 1))
    fi
}

# Test output contains expected string
test_output() {
    local name="$1"
    local pattern="$2"
    shift 2
    local cmd="$*"

    echo -en "${YELLOW}$name${NC} ... "

    local output
    local exit_code
    output=$("$@" 2>&1) && exit_code=0 || exit_code=$?

    if [[ $exit_code -eq 0 ]] && echo "$output" | grep -q "$pattern"; then
        echo -e "${GREEN}PASS${NC}"
        PASS=$((PASS + 1))
    else
        echo -e "${RED}FAIL${NC}"
        log_fail "$name" "$cmd" "Expected output to contain '$pattern'"$'\n'"Exit code: $exit_code"$'\n'"$output"
        FAIL=$((FAIL + 1))
    fi
}

docker_run() {
    docker run --rm --privileged \
        -v "$BIN:/usr/local/bin/pacfetch:ro" \
        -v "/etc/pacman.conf:/etc/pacman.conf:ro" \
        pacfetch-test "$@"
}

# --- Local tests ---
echo -e "\n${YELLOW}=== LOCAL ===${NC}\n"

test_ok "Help (-h)" "$BIN" -h
test_ok "Help (--help)" "$BIN" --help
test_ok "Version (-V)" "$BIN" -V
test_ok "Version (-v)" "$BIN" -v
test_ok "Local flag" "$BIN" --local

test_err "Invalid: -S alone" "$BIN" -S
test_err "Invalid: -y alone" "$BIN" -y
test_err "Invalid: -u alone" "$BIN" -u
test_err "Invalid: -yu" "$BIN" -yu
test_err "Invalid: --badarg" "$BIN" --badarg
test_err "No root: -Sy" "$BIN" -Sy
test_err "No root: -Su" "$BIN" -Su
test_err "No root: -Syu" "$BIN" -Syu

test_output "Disk stat in output" "Disk (/)" "$BIN" --local

# --- Docker tests ---
echo -e "\n${YELLOW}=== DOCKER ===${NC}\n"

if [[ -z "$(docker images -q pacfetch-test 2>/dev/null)" ]]; then
    echo "Building test image..."
    docker build -t pacfetch-test "$SCRIPT_DIR"
fi

test_ok "Docker: --local" docker_run --local
test_ok "Docker: -Sy" docker_run -Sy
test_ok_piped "Docker: -Su (n to prompt)" "n" docker_run -Su
test_ok_piped "Docker: -Syu (n to prompt)" "n" docker_run -Syu



echo -e "\n${YELLOW}=== SUMMARY ===${NC}"
echo -e "Passed: ${GREEN}$PASS${NC}"
echo -e "Failed: ${RED}$FAIL${NC}"

if [[ $FAIL -gt 0 ]]; then
    echo -e "\nSee ${YELLOW}$LOG${NC} for failure details"
fi

[[ $FAIL -eq 0 ]]
