function s = f(x)
    s = 0;
    parfor i = 1:10
        fn = i;
        s = fn(s, x(i));
    end
end
