function f()
    a = zeros(10, 2);
    parfor i = 1:10
        a(i, 1) = 1;
        a(i, 2) = 2;
    end
end
