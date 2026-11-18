function status = h_error_handling(files)
status = zeros(1, numel(files));
for k = 1:numel(files)
    try
        fid = fopen(files{k}, 'r');
        if fid < 0
            error('eh:open', 'cannot open %s', files{k});
        end
        raw = fread(fid, Inf, 'uint8=>char')';
        fclose(fid);
        nums = sscanf(raw, '%f');
        if isempty(nums)
            error('eh:empty', 'no numbers in %s', files{k});
        end
        status(k) = mean(nums);
    catch ME
        switch ME.identifier
            case 'eh:open'
                status(k) = -1;
            case 'eh:empty'
                status(k) = -2;
            otherwise
                rethrow(ME);
        end
        fprintf(2, '%s\n', ME.message);
    end
end
try
    data = load('nonexistent_file.mat');
catch
    data = [];
end
try
    result = status(10);
catch exc
    disp('index failure');
end
w = warning('off', 'all');
cleanup = onCleanup(@() warning(w));
end
