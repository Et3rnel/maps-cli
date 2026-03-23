default_mode := "all"

lint mode=default_mode:
  #!/usr/bin/env sh
  set -eu
  case "{{mode}}" in
    check)
      cargo check --all-targets
      ;;
    clippy)
      cargo clippy --all-targets --all-features -- -D warnings
      ;;
    all)
      cargo check --all-targets
      cargo clippy --all-targets --all-features -- -D warnings
      ;;
    *)
      printf 'Unknown lint mode: %s\nExpected one of: check, clippy, all\n' "{{mode}}" >&2
      exit 1
      ;;
  esac
