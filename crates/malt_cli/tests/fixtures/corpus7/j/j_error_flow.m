function out = j_error_flow(files)
%R3_ERROR_FLOW Process a list of files, collecting errors instead of stopping.
%   OUT.ok and OUT.failed record which files were processed.

out = struct('ok', {{}}, 'failed', {{}});
for k = 1:numel(files)
    f = files{k};
    try
        data = readFile_(f);
        if isempty(data)
            throw(MException('r3:flow:empty', 'File %s is empty', f));
        end
        out.ok{end+1} = f;
    catch ME
        out.failed{end+1} = f;
        switch ME.identifier
            case 'r3:flow:empty'
                fprintf('skip: %s\n', ME.message);
            case {'MATLAB:fileOpen', 'MATLAB:FileIO:InvalidFid'}
                fprintf('cannot open: %s\n', f);
            otherwise
                fprintf('%s\n', getReport(ME, 'basic'));
                rethrow(ME);
        end
    end
end

err = [];
try
    error('Simple:message', 'Value was %d', 42);
catch err
end
if ~isempty(err)
    disp(err.message)
end

cleanup = onCleanup(@() disp('flow finished'));
count = numel(out.ok) + numel(out.failed);
fprintf('%d files\n', count);

    function d = readFile_(name)
        fid = fopen(name, 'r');
        if fid == -1
            error('MATLAB:fileOpen', 'cannot open %s', name);
        end
        d = fread(fid, Inf, 'uint8=>char')';
        fclose(fid);
    end
end
