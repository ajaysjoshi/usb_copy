cargo clean
cargo build --release
mkdir -p ./libs/
cp ./target/release/*.so ./libs/

