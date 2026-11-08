function t5()
    x = 1;
    global g
    y = g;
    persistent p
    function inner()
        persistent q
        global h
    end
end
