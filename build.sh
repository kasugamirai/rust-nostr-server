mkdir output 
cp script/* output 2>/dev/null
cargo build --bin bootstrap --release
cp target/release/bootstrap output
chmod +x output/*
