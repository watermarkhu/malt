function f()
    parfor i = 1:10
        x(i) = i;
        y = x(i, 1);
    end
end
