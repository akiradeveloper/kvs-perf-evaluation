EXE=target/release/rocksdb
for n in 1 10 100 1000 10000 100000 1000000
do
$EXE 4 $n
done