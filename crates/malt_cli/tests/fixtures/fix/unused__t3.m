function t3()
    v = 1;
    function inner()
        v = v + 1;
    end
    n = 0;
    function inn2()
        n = 1;
    end
    inn2();
    inner();
    m = 1;
    f = @() m;
    c = 0;
    while true
        if c > 3, break; end
        c = c + 1;
    end
    error('x');
    disp(1);
    t = 1;
    save('f.mat');
end
