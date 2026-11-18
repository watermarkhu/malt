function f()
    parfor i = 1:10
        for ans = 1:5
            x(i, ans) = i;
        end
    end
end
