function outer()
    spmd
        helper(1);
    end
    function helper(a)
    end
end
