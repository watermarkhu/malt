function pfspmd()
    parfor i = 1:10
        spmd
            x = i;
        end
    end
end
