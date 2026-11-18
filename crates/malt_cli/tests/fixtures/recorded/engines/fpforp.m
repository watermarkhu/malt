function f()
    parfor i = 1:10
        fprintf(fopen('f.txt', 'r'), 'x');
    end
end
