function g2()
    global a b
    c = a;
    function nest()
        a = 1;
    end
    function nest2()
        x = 1;
        if x
            persistent z
        end
    end
end
