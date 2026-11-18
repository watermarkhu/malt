function f()
    parfor i = 1:10
        for j = 1:5
            y = i(j);
        end
    end
end
