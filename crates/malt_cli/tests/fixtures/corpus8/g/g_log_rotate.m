function g_log_rotate(logDir, maxBytes)
%R4_LOG_ROTATE Rotate log files that grow past maxBytes, keeping three copies.

if nargin < 2
    maxBytes = 1e6;
end

files = dir(fullfile(logDir, '*.log'));
rotated = 0;
for f = 1:numel(files)
    if files(f).bytes <= maxBytes
        continue
    end
    src = fullfile(logDir, files(f).name);
    for gen = 3:-1:1
        older = sprintf('%s.%d', src, gen);
        newer = sprintf('%s.%d', src, gen + 1);
        if gen == 3 && exist(older, 'file')
            delete(older)
        elseif exist(older, 'file')
            movefile(older, newer)
        end
    end
    movefile(src, [src '.1']);
    fid = fopen(src, 'w');
    fprintf(fid, 'rotated at %s\n', datestr(now));
    fclose(fid);
    rotated = rotated + 1;
end

exist logDir
exist logDir dir

fprintf('rotated %d of %d files\n', rotated, numel(files))
if rotated == 0, disp('nothing to do'), end

lines = {};
for g = 1:3
    name = sprintf('%s.%d', 'app.log', g);
    lines{g} = name;
end
disp(lines)

try
    cleanupOld(logDir)
catch e
    fprintf(2, 'cleanup failed: %s\n', e.message)
end

    function cleanupOld(d)
        old = dir(fullfile(d, '*.log.4'));
        for k = 1:numel(old)
            delete(fullfile(d, old(k).name))
        end
    end
end
