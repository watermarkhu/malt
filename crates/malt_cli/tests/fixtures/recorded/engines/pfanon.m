function f()
    parfor i = 1:10
        x(i) = i;
        g = @(t) x(i);
    end
end
