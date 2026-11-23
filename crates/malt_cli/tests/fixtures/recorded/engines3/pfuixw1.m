function pfuixw1()
    i = 5;
    a = zeros(1, 10);
    parfor i = 1:10
        a(i) = i;
    end
    disp(i);
end
