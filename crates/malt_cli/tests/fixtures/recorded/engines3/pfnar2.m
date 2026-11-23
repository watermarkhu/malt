function pfnar2(x)
    s = 0;
    parfor i = 1:10
        s = x(i) - s;
    end
end
