function a1(n)
    x = 0;
    x = x + 1;
    disp(x);
    t = 0;
    for i = 1:n
        disp(t);
        t = i;
    end
    cnt = 0;
    while cnt < 3
        cnt = cnt + 1;
    end
    h = @loc;
    h();
    arrayfun(@loc2, 1);
    s.a = 1; s.b = 2;
    disp(s);
    p.q = 1;
    p.q;
    z = zeros(1,n);
    z(1) = 5;
    [aa, bb] = size(ones(2));
    disp(aa);
    1;
    'str';
    x ~= 2;
    x > 2;
    x(1);
    x';
    ans;
    ~x;
    if x, y = 1; end
end
function loc(), end
function loc2(~), end
function unusedLocal(), end
