# use `sh build-on-termux.sh` to execute this.

cargo b
path=$(grep "target-dir" ./.cargo/config.toml | cut -d'=' -f2 | tr -d ' ' | tr -d '"')
rsync -av --delete $path/ ./target/

