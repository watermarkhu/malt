function f()
    parfor i = 1:10
        fid = fopen('f.txt', 'r');
        fwrite(fid, 1);
    end
end
