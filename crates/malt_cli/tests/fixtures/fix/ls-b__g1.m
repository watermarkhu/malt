function g1()
    g = 3;
    global g
    p = p + 1;
    persistent p
    if true
        global q
    end
    q = 1;
    global q
    global r
    r = 2;
end
