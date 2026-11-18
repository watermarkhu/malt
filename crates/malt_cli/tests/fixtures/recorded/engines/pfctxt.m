function f()
    parfor i = 1:10
        for j = 1:5
            x(i, j) = i * j;
        end
        y(i) = x(i, j);
    end
end
