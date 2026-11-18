function foo()
    parfor i = 1:10
        parfor j = 1:5
            x = i + j;
        end
    end
end
