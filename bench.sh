for k in 1 4 16
do
    for n in 1 10 100 1000 10000 100000 1000000
    do
        echo RUN $k $n
        $EXE $k $n
    done
done